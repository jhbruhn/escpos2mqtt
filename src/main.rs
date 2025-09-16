use env_logger;
use envconfig::Envconfig;
use escpos::driver::NetworkDriver;

use mqtt_typed_client::MqttClient;
use mqtt_typed_client_macros::mqtt_topic;
use uuid::Uuid;

mod printer;
mod program;
mod string_serializer;

use crate::program::Program;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "PRINTER_HOST")]
    pub printer_host: String,

    #[envconfig(from = "PRINTER_NAME", default = "printer")]
    pub printer_name: String,

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

    log::info!("Connecting to printer at {}", config.printer_host);
    let driver = NetworkDriver::open(&config.printer_host, 9100, None)?;
    let mut printer = printer::Printer::new(driver);
    log::info!("Connected to printer at {}", config.printer_host);

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
                continue;
            }

            let program_string = job.payload;
            if let Ok((_remains, program)) = Program::parse(&program_string) {
                log::info!("Printing program {:?}", program);
                printer.print(&program)?;
            }
        } else {
            log::error!("Could not parse message: {:?}", response);
        }
    }
}
