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
pub struct Configuration {
    name: String,
    command_topic: String,
    availability_topic: String,
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
        availability_topic: &str,
        unique_id: &str,
        device_id: &str,
        device_name: &str,
        device_model: &str,
    ) -> Configuration {
        Configuration {
            name: String::from(name),
            command_topic: String::from(command_topic),
            availability_topic: String::from(availability_topic),
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
