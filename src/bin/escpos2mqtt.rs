use env_logger;
use envconfig::Envconfig;
use mqtt_typed_client::{MqttClient, MqttClientConfig};
use std::time::Duration;
use uuid::Uuid;

use escpos2mqtt::discovery_service::{DiscoveryConfig, DiscoveryService};
use escpos2mqtt::mqtt_service::MqttService;
use escpos2mqtt::mqtt::topics::ServiceAvailableTopic;
use escpos2mqtt::mqtt::topics::service_available_topic::ServiceAvailableTopicExt;
use escpos2mqtt::registry::PrinterRegistry;

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

    #[envconfig(from = "DISCOVERY_INTERVAL_SECS", default = "30")]
    pub discovery_interval_secs: u64,

    #[envconfig(from = "PRINTER_TIMEOUT_SECS", default = "60")]
    pub printer_timeout_secs: u64,
}

#[allow(dead_code)]
pub fn get_client_id(prefix: &str) -> String {
    let uuid = Uuid::new_v4().to_string();
    let short_uuid = &uuid[..8];
    format!("{prefix}_{short_uuid}")
}

#[allow(dead_code)]
pub fn build_url(base_url: &str, client_id_prefix: &str) -> String {
    let client_id = get_client_id(client_id_prefix);

    if base_url.contains('?') {
        format!("{base_url}&client_id={client_id}")
    } else {
        format!("{base_url}?client_id={client_id}")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::init_from_env().unwrap();

    log::info!("Starting escpos2mqtt");

    // Connect to MQTT Broker
    log::info!("Connecting to MQTT Broker");

    let mut mqtt_config =
        MqttClientConfig::<escpos2mqtt::mqtt::string_serializer::JsonSerializer>::from_url(
            &build_url(&config.mqtt_url, "escpos"),
        )?;

    let last_will = ServiceAvailableTopic::last_will(String::from("offline"))
        .qos(mqtt_typed_client::QoS::AtLeastOnce);

    mqtt_config.with_last_will(last_will)?;

    let (client, connection) =
        MqttClient::<escpos2mqtt::mqtt::string_serializer::JsonSerializer>::connect_with_config(
            mqtt_config,
        )
        .await?;

    // Publish online status
    let online_topic = client.service_available_topic();
    online_topic.publish(&"online".to_string()).await?;

    log::info!("Connected to MQTT broker");

    // Create shared printer registry
    let registry = PrinterRegistry::new();

    // Subscribe to registry events
    let registry_event_rx = registry.subscribe();

    // Add manual printer to registry if configured (will emit event automatically)
    if let Some(host) = &config.printer_host {
        const MANUAL_PRINTER_ID: &str = "manual";

        let host_clone = host.clone();
        let mut manual_printer = escpos2mqtt::printer::Printer::new(
            move || {
                log::debug!("Connecting to printer at {}", &host_clone);
                escpos::driver::NetworkDriver::open(&host_clone, 9100, None)
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
            .unwrap_or(config.default_printer_model.clone());

        log::info!(
            "Adding manually configured printer with id {}, and model {}",
            MANUAL_PRINTER_ID,
            &manual_printer_model_config,
        );

        let manual_profile = escpos_db::ALL_PROFILES
            .get(&manual_printer_model_config)
            .expect(&format!(
                "Printer model {} not found!",
                &manual_printer_model_config
            ));

        registry.add_manual_printer(
            MANUAL_PRINTER_ID.to_string(),
            manual_printer,
            *manual_profile,
        ).await;
    }

    // Create discovery service config
    let discovery_config = DiscoveryConfig {
        default_printer_model: config.default_printer_model,
        discovery_interval: Duration::from_secs(config.discovery_interval_secs),
        printer_timeout: Duration::from_secs(config.printer_timeout_secs),
    };

    // Clone client and registry for services
    let mqtt_service_client = client.clone();
    let discovery_registry = registry.clone();
    let mqtt_service_registry = registry.clone();

    // Create services
    let discovery_service = DiscoveryService::new(discovery_config, discovery_registry);

    let mqtt_service = MqttService::new(
        mqtt_service_registry,
        mqtt_service_client,
        registry_event_rx,
    );

    log::info!(
        "Starting discovery service (interval: {}s)",
        config.discovery_interval_secs
    );
    log::info!("Starting MQTT service (handles all MQTT operations)");

    // Spawn discovery service
    let discovery_handle = tokio::spawn(async move {
        if let Err(e) = discovery_service.run().await {
            log::error!("Discovery service error: {}", e);
        }
    });

    // Spawn MQTT service
    let mqtt_handle = tokio::spawn(async move {
        if let Err(e) = mqtt_service.run().await {
            log::error!("MQTT service error: {}", e);
        }
    });

    log::info!("All services started successfully");

    // Keep the connection alive by not dropping it
    let _connection = connection;

    // Wait for all tasks to complete (they shouldn't under normal operation)
    let result = tokio::try_join!(discovery_handle, mqtt_handle);

    if let Err(e) = result {
        log::error!("Service error: {}", e);
    }

    Ok(())
}
