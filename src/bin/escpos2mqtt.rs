use env_logger;
use envconfig::Envconfig;
use escpos::driver::NetworkDriver;
use mqtt_typed_client::MqttClient;
use mqtt_typed_client_macros::mqtt_topic;
use std::collections::HashMap;
use uuid::Uuid;

use escpos2mqtt::*;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "PRINTER_HOST")]
    pub printer_host: String,

    #[envconfig(from = "PRINTER_MODEL", default = "default")]
    pub printer_model: String,

    #[envconfig(from = "MQTT_URL")]
    pub mqtt_url: String,
}

#[mqtt_topic("escpos/{printer}/print")]
#[derive(Debug)]
pub struct PrintJobTopic {
    printer: String, // Extracted from first topic parameter {language}
    payload: String, // Automatically deserialized message payload
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::init_from_env().unwrap();

    let default_profile = escpos_db::ALL_PROFILES
        .get(&config.printer_model)
        .expect(&format!(
            "Printer model {} not found!",
            &config.printer_model
        ));

    let mut printers = HashMap::new();

    const MANUAL_PRINTER_ID: &str = "manual";

    log::info!(
        "Adding manually configured printer with id {}, host {} and model {}",
        MANUAL_PRINTER_ID,
        &config.printer_host,
        &config.printer_model,
    );
    printers.insert(
        MANUAL_PRINTER_ID.to_string(),
        (
            printer::Printer::new(
                move || {
                    log::info!("Connecting to printer at {}", config.printer_host);
                    NetworkDriver::open(&config.printer_host, 9100, None)
                },
                "Manual Printer",
                &format!(
                    "Manually configured printer of type {}",
                    config.printer_model
                ),
            ),
            default_profile,
        ),
    );

    let discovered_printers = printer::discover_network().await?;

    for discovered_printer in discovered_printers {
        let id = discovered_printer.name.to_lowercase();

        log::info!(
            "Adding network-discovered printer with id {} and model {}",
            id,
            default_profile.name
        );
        printers.insert(id, (discovered_printer, default_profile));
    }

    log::info!("Connecting to MQTT Broker.");
    let (client, _connection) = MqttClient::<string_serializer::JsonSerializer>::connect(
        &build_url(&config.mqtt_url, "escpos"),
    )
    .await?;

    let topic_client = client.print_job_topic();

    let mut subscriber = topic_client.subscribe().await?;

    for id in printers.keys() {
        log::info!("Listening on escpos/{}/print", id);
    }

    loop {
        let response = subscriber.receive().await;

        if let Some(Ok(job)) = response {
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
            log::error!("Could not parse message: {:?}", response);
        }
    }
}
