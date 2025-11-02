use mqtt_typed_client_macros::mqtt_topic;

#[mqtt_topic("escpos/available")]
pub struct ServiceAvailableTopic {
    pub payload: String,
}

#[mqtt_topic("escpos/{printer}/print")]
#[derive(Debug)]
pub struct PrintJobTopic {
    pub printer: String,
    pub payload: String,
}

#[mqtt_topic("homeassistant/{domain}/{id}/config")]
#[derive(Debug)]
pub struct HomeAssistantDiscoveryTopic {
    pub domain: String,
    pub id: String,
    pub payload: Option<crate::mqtt::homeassistant::Configuration>,
}
