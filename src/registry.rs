use escpos_db::Profile;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::printer::Printer;

/// Metadata about a printer's lifecycle in the registry
#[derive(Debug, Clone)]
pub struct PrinterMetadata {
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub is_manual: bool,
}

impl PrinterMetadata {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            first_seen: now,
            last_seen: now,
            is_manual: false,
        }
    }

    pub fn new_manual() -> Self {
        let now = SystemTime::now();
        Self {
            first_seen: now,
            last_seen: now,
            is_manual: true,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
}

/// Entry in the printer registry containing the printer, profile, and metadata
#[derive(Debug)]
pub struct PrinterEntry {
    pub printer: Printer,
    pub profile: &'static Profile<'static>,
    pub metadata: PrinterMetadata,
}

/// Event emitted when a printer is added to the registry
#[derive(Debug, Clone)]
pub struct PrinterAddedEvent {
    pub printer_id: String,
    pub printer_name: String,
    pub printer_description: String,
    pub model_name: String,
}

/// Event emitted when a printer is removed from the registry
#[derive(Debug, Clone)]
pub struct PrinterRemovedEvent {
    pub printer_id: String,
}

/// Registry events sent through channel
#[derive(Debug, Clone)]
pub enum RegistryEvent {
    Added(PrinterAddedEvent),
    Removed(PrinterRemovedEvent),
}

/// Thread-safe registry for managing discovered printers
#[derive(Clone)]
pub struct PrinterRegistry {
    printers: Arc<RwLock<HashMap<String, PrinterEntry>>>,
    event_tx: broadcast::Sender<RegistryEvent>,
}

impl PrinterRegistry {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            printers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Subscribe to registry events
    pub fn subscribe(&self) -> broadcast::Receiver<RegistryEvent> {
        self.event_tx.subscribe()
    }

    /// Add or update a printer in the registry
    pub async fn add_printer(
        &self,
        id: String,
        printer: Printer,
        profile: &'static Profile<'static>,
    ) {
        self.add_printer_internal(id, printer, profile, false).await;
    }

    /// Add a manual printer to the registry (won't timeout)
    pub async fn add_manual_printer(
        &self,
        id: String,
        printer: Printer,
        profile: &'static Profile<'static>,
    ) {
        self.add_printer_internal(id, printer, profile, true).await;
    }

    /// Internal method to add a printer with configurable manual flag
    async fn add_printer_internal(
        &self,
        id: String,
        printer: Printer,
        profile: &'static Profile<'static>,
        is_manual: bool,
    ) {
        let mut printers = self.printers.write().await;

        if let Some(entry) = printers.get_mut(&id) {
            // Update existing printer's last_seen
            entry.metadata.update_last_seen();
            log::debug!("Updated existing printer: {}", id);
        } else {
            // Add new printer
            let printer_name = printer.name.clone();
            let printer_description = printer.description.clone();
            let model_name = profile.name.to_string();

            let metadata = if is_manual {
                PrinterMetadata::new_manual()
            } else {
                PrinterMetadata::new()
            };

            printers.insert(
                id.clone(),
                PrinterEntry {
                    printer,
                    profile,
                    metadata,
                },
            );
            log::info!("Added new printer to registry: {} (manual: {})", id, is_manual);

            // Emit added event for new printers
            let event = PrinterAddedEvent {
                printer_id: id.clone(),
                printer_name,
                printer_description,
                model_name,
            };
            let _ = self.event_tx.send(RegistryEvent::Added(event));
        }
    }

    /// Update last_seen timestamp for an existing printer
    pub async fn touch_printer(&self, id: &str) {
        let mut printers = self.printers.write().await;
        if let Some(entry) = printers.get_mut(id) {
            entry.metadata.update_last_seen();
            log::debug!("Touched printer: {}", id);
        }
    }

    /// Remove a printer from the registry
    pub async fn remove_printer(&self, id: &str) -> Option<PrinterEntry> {
        let mut printers = self.printers.write().await;
        let removed = printers.remove(id);
        if removed.is_some() {
            log::info!("Removed printer from registry: {}", id);

            // Emit removed event
            let event = PrinterRemovedEvent {
                printer_id: id.to_string(),
            };
            let _ = self.event_tx.send(RegistryEvent::Removed(event));
        }
        removed
    }

    /// Get a mutable reference to a printer (for printing)
    pub async fn get_printer_mut(&self, id: &str) -> Option<Printer> {
        let printers = self.printers.read().await;
        // We need to return a clone because we can't return a mutable reference
        // through the RwLock. Fortunately, Printer contains an UnboundedSender
        // which is Clone, so this is cheap.
        printers.get(id).map(|entry| Printer {
            name: entry.printer.name.clone(),
            description: entry.printer.description.clone(),
            program_sender: entry.printer.program_sender.clone(),
        })
    }

    /// Get printer with profile (for read-only operations)
    pub async fn get_printer_with_profile(
        &self,
        id: &str,
    ) -> Option<(Printer, &'static Profile<'static>)> {
        let printers = self.printers.read().await;
        printers.get(id).map(|entry| {
            (
                Printer {
                    name: entry.printer.name.clone(),
                    description: entry.printer.description.clone(),
                    program_sender: entry.printer.program_sender.clone(),
                },
                entry.profile,
            )
        })
    }

    /// List all printer IDs and their names
    pub async fn list_printers(&self) -> Vec<(String, String)> {
        let printers = self.printers.read().await;
        printers
            .iter()
            .map(|(id, entry)| (id.clone(), entry.printer.name.clone()))
            .collect()
    }

    /// Get all printers with their profiles (useful for discovery publishing)
    pub async fn get_all_printers(&self) -> Vec<(String, Printer, &'static Profile<'static>)> {
        let printers = self.printers.read().await;
        printers
            .iter()
            .map(|(id, entry)| {
                (
                    id.clone(),
                    Printer {
                        name: entry.printer.name.clone(),
                        description: entry.printer.description.clone(),
                        program_sender: entry.printer.program_sender.clone(),
                    },
                    entry.profile,
                )
            })
            .collect()
    }

    /// Detect changes between current registry and new printer set
    /// Returns (newly_added, still_present)
    pub async fn diff(
        &self,
        new_printers: &HashMap<String, (Printer, &'static Profile<'static>)>,
    ) -> (Vec<String>, Vec<String>) {
        let printers = self.printers.read().await;
        let current_ids: std::collections::HashSet<_> = printers.keys().cloned().collect();
        let new_ids: std::collections::HashSet<_> = new_printers.keys().cloned().collect();

        let newly_added: Vec<String> = new_ids.difference(&current_ids).cloned().collect();
        let still_present: Vec<String> = new_ids.intersection(&current_ids).cloned().collect();

        (newly_added, still_present)
    }

    /// Get count of registered printers
    pub async fn count(&self) -> usize {
        let printers = self.printers.read().await;
        printers.len()
    }

    /// Get IDs of printers that haven't been seen within the timeout duration
    /// Manual printers are excluded from this check
    pub async fn get_stale_printers(&self, timeout: std::time::Duration) -> Vec<String> {
        let printers = self.printers.read().await;
        let now = SystemTime::now();

        printers
            .iter()
            .filter_map(|(id, entry)| {
                // Skip manual printers - they never timeout
                if entry.metadata.is_manual {
                    return None;
                }

                if let Ok(elapsed) = now.duration_since(entry.metadata.last_seen) {
                    if elapsed > timeout {
                        Some(id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
