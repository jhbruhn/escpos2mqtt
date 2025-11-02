use env_logger;
use envconfig::Envconfig;
use escpos::driver::NetworkDriver;
use escpos_db::Profile;
use mqtt_typed_client::{MqttClient, MqttClientConfig};
use mqtt_typed_client_macros::mqtt_topic;
use std::collections::HashMap;
use uuid::Uuid;

use escpos2mqtt::*;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "MANUAL_PRINTER_HOST")]
    pub printer_host: Option<String>,

    #[envconfig(from = "MANUAL_PRINTER_MODEL")]
    pub printer_model: Option<String>,

    #[envconfig(from = "DEFAULT_PRINTER_MODEL", default = "default")]
    pub default_printer_model: String,

    #[envconfig(from = "MQTT_URL")]
    pub mqtt_url: String,
}

#[mqtt_topic("escpos/{printer}/print")]
#[derive(Debug)]
pub struct PrintJobTopic {
    printer: String, // Extracted from first topic parameter {language}
    payload: String, // Automatically deserialized message payload
}

#[mqtt_topic("homeassistant/{domain}/{id}/config")]
#[derive(Debug)]
pub struct HomeAssistantDiscoveryTopic {
    domain: String,
    id: String,
    payload: mqtt::homeassistant::Configuration,
}

#[mqtt_topic("escpos/available")]
pub struct ServiceAvailableTopic {
    payload: String,
}

#[allow(dead_code)]
pub fn get_client_id(prefix: &str) -> String {
    let uuid = Uuid::new_v4().to_string();
    let short_uuid = &uuid[..8]; // Take first 8 characters
    format!("{prefix}_{short_uuid}")
}

#[allow(dead_code)]
pub fn build_url(base_url: &str, client_id_prefix: &str) -> String {
    let client_id = get_client_id(client_id_prefix);

    if base_url.contains('?') {
        // URL already has query parameters, append with &
        format!("{base_url}&client_id={client_id}")
    } else {
        // URL has no query parameters, start with ?
        format!("{base_url}?client_id={client_id}")
    }
}

async fn get_printers(
    config: &Config,
) -> anyhow::Result<HashMap<String, (printer::Printer, &Profile<'static>)>> {
    let mut printers = HashMap::new();

    if let Some(host) = config.printer_host.clone() {
        const MANUAL_PRINTER_ID: &str = "manual";

        let mut manual_printer = printer::Printer::new(
            move || {
                log::debug!("Connecting to printer at {}", &host);
                NetworkDriver::open(&host, 9100, None)
            },
            "Manual Printer",
            "Manually configured printer",
        );
        let manual_model_name = manual_printer.model_name().await;

        if let Some(overrider) = &config.printer_model {
            if let Ok(manual_model_name) = &manual_model_name {
                if manual_model_name != overrider {
                    log::warn!(
                        "Overriding manual printer type with {} (actual type is {})",
                        overrider,
                        manual_model_name
                    );
                }
            }
        }
        let manual_printer_model_config = config
            .printer_model
            .clone()
            .or(manual_printer.model_name().await.ok())
            .unwrap_or("default".to_string());

        log::debug!(
            "Adding manually configured printer with id {}, and model {}",
            MANUAL_PRINTER_ID,
            &manual_printer_model_config,
        );
        let default_profile = escpos_db::ALL_PROFILES
            .get(&manual_printer_model_config)
            .expect(&format!(
                "Printer model {} not found!",
                &manual_printer_model_config
            ));

        printers.insert(
            MANUAL_PRINTER_ID.to_string(),
            (manual_printer, (*default_profile)),
        );
    }

    let discovered_printers = printer::discover_network().await?;

    for mut discovered_printer in discovered_printers {
        let id = discovered_printer.name.to_lowercase();

        let model_name = discovered_printer
            .model_name()
            .await
            .ok()
            .unwrap_or(config.default_printer_model.clone());

        let profile = escpos_db::ALL_PROFILES.get(&model_name).unwrap();
        log::debug!(
            "Adding network-discovered printer with id {} and model {:?}",
            id,
            &profile.name
        );
        printers.insert(id, (discovered_printer, (*profile)));
    }

    Ok(printers)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::init_from_env().unwrap();

    log::info!("Running printer discovery");
    let printers = get_printers(&config).await;
    if let Ok(printers) = printers {
        log::info!("Printer discovery finished, discovered printers:");
        for entry in printers.iter() {
            log::info!("Printer {}: {}", entry.0, entry.1 .1.name);
        }
        if printers.len() == 0 {
            log::info!("No printers discovered!");
        }
    } else if let Err(err) = printers {
        log::error!("Could not discovery printers, {}", err);
    }

    log::info!("Connecting to MQTT Broker.");

    let mut mqtt_config = MqttClientConfig::<mqtt::string_serializer::JsonSerializer>::from_url(
        &build_url(&config.mqtt_url, "escpos"),
    )?;

    let last_will = ServiceAvailableTopic::last_will(String::from("offline"))
        .qos(mqtt_typed_client::QoS::AtLeastOnce);

    mqtt_config.with_last_will(last_will)?;

    let (client, _connection) =
        MqttClient::<mqtt::string_serializer::JsonSerializer>::connect_with_config(mqtt_config)
            .await?;

    let online_topic = client.service_available_topic();
    online_topic.publish(&"online".to_string()).await?;

    for (id, (printer, profile)) in get_printers(&config).await? {
        let message = mqtt::homeassistant::Configuration::new(
            mqtt::homeassistant::Domain::Notify,
            "Receipt",
            &format!("escpos/{}/print", id),
            "escpos/available",
            &id,
            &id,
            &printer.name,
            &format!("{} - {}", &profile.name, &printer.description),
        );

        client
            .home_assistant_discovery_topic()
            .publish(
                &mqtt::homeassistant::Domain::Notify.to_string(),
                &id,
                &message,
            )
            .await?;
    }

    let topic_client = client.print_job_topic();

    let mut subscriber = topic_client.subscribe().await?;

    log::info!("Listening for print jobs");

    loop {
        let response = subscriber.receive().await;

        if let Some(Ok(job)) = response {
            let printers = get_printers(&config).await;
            if let Ok(mut printers) = printers {
                if let Some((printer, profile)) = printers.get_mut(job.printer.as_str()) {
                    let program_string = job.payload;
                    let parsed = program::Program::parse(&program_string);
                    if let Ok((remains, program)) = parsed {
                        if remains.len() > 0 {
                            log::error!(
                                "Could not fully parse program. Failed to parse from: {}",
                                remains
                            )
                        } else {
                            log::info!("Printing program {:?}", program);

                            if let Err(err) = printer
                                .print(renderer::render(program, profile).await)
                                .await
                            {
                                log::error!("Failed to print: {}", err);
                            } else {
                                log::info!("Printed program.")
                            }
                        }
                    } else if let Err(err) = parsed {
                        log::error!("Could not parse program: {}", err);
                    }
                } else {
                    log::error!("Could not find printer with id {}", job.printer);
                }
            } else {
                log::error!("Could not discover printers: {}", printers.unwrap_err());
            }
        } else {
            log::error!("Could not parse message: {:?}", response);
        }
    }
}
