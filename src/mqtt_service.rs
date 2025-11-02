use crate::mqtt::homeassistant;
use crate::mqtt::topics::home_assistant_discovery_topic::HomeAssistantDiscoveryTopicExt;
use crate::mqtt::topics::print_job_topic::PrintJobTopicExt;
use crate::program;
use crate::registry::PrinterRegistry;
use crate::registry::RegistryEvent;
use crate::renderer;
use mqtt_typed_client::MqttClient;
use tokio::sync::broadcast;

pub struct MqttService {
    registry: PrinterRegistry,
    client: MqttClient<crate::mqtt::string_serializer::JsonSerializer>,
    registry_event_rx: broadcast::Receiver<RegistryEvent>,
}

impl MqttService {
    pub fn new(
        registry: PrinterRegistry,
        client: MqttClient<crate::mqtt::string_serializer::JsonSerializer>,
        registry_event_rx: broadcast::Receiver<RegistryEvent>,
    ) -> Self {
        Self {
            registry,
            client,
            registry_event_rx,
        }
    }

    /// Run the MQTT service in a loop, handling all MQTT operations
    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("MQTT service listening for print jobs and events");

        // Subscribe to print job topic
        let topic_client = self.client.print_job_topic();
        let mut subscriber = topic_client.subscribe().await?;

        loop {
            tokio::select! {
                // Handle incoming print jobs
                response = subscriber.receive() => {
                    if let Some(result) = response {
                        match result {
                            Ok(topic) => {
                                self.handle_print_job(&topic.printer, &topic.payload).await;
                            }
                            Err(e) => {
                                log::error!("Could not parse MQTT message: {:?}", e);
                            }
                        }
                    } else {
                        log::error!("MQTT subscriber returned None");
                    }
                }

                // Handle registry events
                Ok(event) = self.registry_event_rx.recv() => {
                    self.handle_registry_event(event).await;
                }
            }
        }
    }

    /// Handle registry events
    async fn handle_registry_event(&self, event: RegistryEvent) {
        match event {
            RegistryEvent::Added(e) => {
                log::info!(
                    "Publishing HA discovery for printer: {} ({})",
                    e.printer_id,
                    e.printer_name
                );

                if let Err(err) = self
                    .publish_discovery(
                        &e.printer_id,
                        &e.printer_name,
                        &e.printer_description,
                        &e.model_name,
                    )
                    .await
                {
                    log::error!("Failed to publish HA discovery: {}", err);
                }
            }
            RegistryEvent::Removed(e) => {
                log::info!("Publishing HA removal for printer: {}", e.printer_id);

                if let Err(err) = self.publish_removal(&e.printer_id).await {
                    log::error!("Failed to publish HA removal: {}", err);
                }
            }
        }
    }

    /// Handle a single print job
    async fn handle_print_job(&self, printer_id: &str, payload: &str) {
        log::info!("Received print job for printer: {}", printer_id);

        // Look up printer in registry
        let printer_result = self.registry.get_printer_with_profile(printer_id).await;

        if let Some((mut printer, profile)) = printer_result {
            // Parse the program
            let parsed = program::Program::parse(payload);

            match parsed {
                Ok((remains, program)) => {
                    if !remains.is_empty() {
                        log::error!(
                            "Could not fully parse program. Failed to parse from: {}",
                            remains
                        );
                    } else {
                        log::info!("Printing program {:?}", program);

                        // Render and print
                        match printer
                            .print(renderer::render(program, profile).await)
                            .await
                        {
                            Ok(_) => {
                                log::info!("Successfully printed to printer: {}", printer_id);
                            }
                            Err(err) => {
                                log::error!("Failed to print to {}: {}", printer_id, err);
                            }
                        }
                    }
                }
                Err(err) => {
                    log::error!("Could not parse program: {}", err);
                }
            }
        } else {
            log::error!(
                "Printer '{}' not found in registry. Available printers: {:?}",
                printer_id,
                self.registry.list_printers().await
            );
        }
    }

    /// Publish Home Assistant discovery message for a printer
    async fn publish_discovery(
        &self,
        printer_id: &str,
        printer_name: &str,
        printer_description: &str,
        model_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = homeassistant::Configuration::new(
            homeassistant::Domain::Notify,
            "Receipt",
            &format!("escpos/{}/print", printer_id),
            "escpos/available",
            printer_id,
            printer_id,
            printer_name,
            &format!("{} - {}", model_name, printer_description),
        );

        self.client
            .home_assistant_discovery_topic()
            .publish(
                &homeassistant::Domain::Notify.to_string(),
                printer_id,
                &Some(message),
            )
            .await?;

        Ok(())
    }

    /// Publish Home Assistant removal message for a printer
    async fn publish_removal(&self, printer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .home_assistant_discovery_topic()
            .publish(
                &homeassistant::Domain::Notify.to_string(),
                printer_id,
                &None,
            )
            .await?;

        Ok(())
    }
}
