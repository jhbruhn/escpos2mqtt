use crate::printer;
use crate::registry::PrinterRegistry;
use escpos_db::Profile;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;

pub struct DiscoveryConfig {
    pub default_printer_model: String,
    pub discovery_interval: Duration,
    pub printer_timeout: Duration,
}

pub struct DiscoveryService {
    config: DiscoveryConfig,
    registry: PrinterRegistry,
}

impl DiscoveryService {
    pub fn new(config: DiscoveryConfig, registry: PrinterRegistry) -> Self {
        Self { config, registry }
    }

    /// Run the discovery service in a loop
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tick = interval(self.config.discovery_interval);

        loop {
            tick.tick().await;
            log::info!("Running periodic printer discovery");
            log::debug!("About to call discover_and_update");

            if let Err(e) = self.discover_and_update().await {
                log::error!("Discovery failed: {}", e);
            }
        }
    }

    /// Discover printers and update the registry
    async fn discover_and_update(&self) -> anyhow::Result<()> {
        log::debug!("discover_and_update: calling get_printers");
        let mut printers = self.get_printers().await?;
        log::debug!("discover_and_update: got {} printers", printers.len());

        // Diff against current registry
        let (newly_added, still_present) = self.registry.diff(&printers).await;

        // Only add NEW printers to registry (registry will emit events)
        for id in &newly_added {
            if let Some((printer, profile)) = printers.remove(id) {
                self.registry
                    .add_printer(id.clone(), printer, profile)
                    .await;
            }
        }

        // For still-present printers, just touch their last_seen timestamp
        // Don't replace the Printer instance (which would spawn new tasks)
        for id in &still_present {
            self.registry.touch_printer(id).await;
        }

        // Check for disappeared printers (not seen beyond timeout)
        let disappeared_ids = self
            .registry
            .get_stale_printers(self.config.printer_timeout)
            .await;

        // Remove from registry (registry will emit events)
        for id in disappeared_ids {
            log::info!("Printer {} has disappeared (timeout)", id);
            self.registry.remove_printer(&id).await;
        }

        log::debug!(
            "Discovery complete: {} new, {} existing",
            newly_added.len(),
            still_present.len()
        );

        Ok(())
    }

    /// Get all printers (network discovered only)
    /// Manual printers are added directly to the registry on startup
    async fn get_printers(
        &self,
    ) -> anyhow::Result<HashMap<String, (printer::Printer, &'static Profile<'static>)>> {
        log::debug!("get_printers: starting");
        let mut printers = HashMap::new();

        // Discover network printers
        log::debug!("get_printers: discovering network printers");
        let discovered_printers = printer::discover_network().await?;
        log::debug!(
            "get_printers: found {} network printers",
            discovered_printers.len()
        );

        for mut discovered_printer in discovered_printers {
            let id = discovered_printer.name.to_lowercase();

            let model_name = discovered_printer
                .model_name()
                .await
                .ok()
                .unwrap_or(self.config.default_printer_model.clone());

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
}
