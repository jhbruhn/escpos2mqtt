use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub enum Domain {
    #[default]
    Notify,
}

impl ToString for Domain {
    fn to_string(&self) -> String {
        String::from(match self {
            Domain::Notify => "notify",
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Device {
    identifiers: Vec<String>,
    name: String,
    model: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvailabilityEntry {
    pub topic: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    name: String,
    command_topic: String,
    availability: Vec<AvailabilityEntry>,
    availability_mode: String,
    unique_id: String,
    device: Device,
    #[serde(skip)]
    _domain: Domain,
}

impl Configuration {
    pub fn new(
        domain: Domain,
        name: &str,
        command_topic: &str,
        service_availability_topic: &str,
        printer_availability_topic: &str,
        unique_id: &str,
        device_id: &str,
        device_name: &str,
        device_model: &str,
    ) -> Configuration {
        Configuration {
            name: String::from(name),
            command_topic: String::from(command_topic),
            availability: vec![
                AvailabilityEntry {
                    topic: String::from(service_availability_topic),
                },
                AvailabilityEntry {
                    topic: String::from(printer_availability_topic),
                },
            ],
            availability_mode: String::from("all"),
            unique_id: String::from(unique_id),
            device: Device {
                name: String::from(device_name),
                identifiers: vec![String::from(device_id)],
                model: String::from(device_model),
            },
            _domain: domain,
        }
    }

    pub fn configuration_topic(&self) -> String {
        format!(
            "homeassistant/{}/{}/config",
            self._domain.to_string(),
            self.unique_id
        )
    }
}
