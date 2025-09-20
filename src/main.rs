use env_logger;
use envconfig::Envconfig;
use escpos::driver::NetworkDriver;

use mqtt_typed_client::MqttClient;
use mqtt_typed_client_macros::mqtt_topic;
use uuid::Uuid;

mod mini_crossword;
mod printer;
mod program;
mod renderer;
mod string_serializer;

use crate::program::Program;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "PRINTER_HOST")]
    pub printer_host: String,

    #[envconfig(from = "PRINTER_NAME", default = "printer")]
    pub printer_name: String,

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

    let printer_profile = escpos_db::ALL_PROFILES
        .get(&config.printer_model)
        .expect(&format!(
            "Printer model {} not found!",
            &config.printer_model
        ));

    let mut printer = printer::Printer::new(move || {
        log::info!("Connecting to printer at {}", config.printer_host);
        NetworkDriver::open(&config.printer_host, 9100, None)
    });

    log::info!("Connecting to MQTT Broker.");
    let (client, _connection) = MqttClient::<string_serializer::JsonSerializer>::connect(
        &build_url(&config.mqtt_url, "escpos"),
    )
    .await?;

    let topic_client = client.print_job_topic();

    let mut subscriber = topic_client.subscribe().await?;

    log::info!("Listening on escpos/{}/print", config.printer_name);

    loop {
        let response = subscriber.receive().await;

        if let Some(Ok(job)) = response {
            if job.printer != config.printer_name {
                log::info!("{} is not us ({})", job.printer, config.printer_name);
                continue;
            }

            let program_string = job.payload;
            let parsed = Program::parse(&program_string);
            if let Ok((remains, program)) = parsed {
                if remains.len() > 0 {
                    log::error!(
                        "Could not fully parse program. Failed to parse from: {}",
                        remains
                    )
                } else {
                    log::info!("Printing program {:?}", program);

                    if let Err(err) = printer
                        .print(renderer::render(program, printer_profile).await)
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
            log::error!("Could not parse message: {:?}", response);
        }
    }
}
