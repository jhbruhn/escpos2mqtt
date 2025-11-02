use mqtt_typed_client::MessageSerializer;

use crate::mqtt::homeassistant::Configuration;

#[derive(Clone, Default)]
pub struct JsonSerializer;

//impl<T> MessageSerializer<T> for JsonSerializer
//where
//    T: Serialize + DeserializeOwned + 'static,
//{
//    type SerializeError = serde_json::Error;
//    type DeserializeError = serde_json::Error;
//
//    fn serialize(&self, data: &T) -> Result<Vec<u8>, Self::SerializeError> {
//        serde_json::to_vec(data)
//    }
//
//    fn deserialize(&self, bytes: &[u8]) -> Result<T, Self::DeserializeError> {
//        serde_json::from_slice(bytes)
//    }
//}

impl MessageSerializer<String> for JsonSerializer {
    type SerializeError = serde_json::Error;
    type DeserializeError = std::str::Utf8Error;

    fn serialize(&self, data: &String) -> Result<Vec<u8>, Self::SerializeError> {
        Ok(data.clone().into_bytes())
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<String, Self::DeserializeError> {
        Ok(String::from(str::from_utf8(bytes)?))
    }
}

impl MessageSerializer<Vec<u8>> for JsonSerializer {
    type SerializeError = serde_json::Error;
    type DeserializeError = serde_json::Error;

    fn serialize(&self, data: &Vec<u8>) -> Result<Vec<u8>, Self::SerializeError> {
        serde_json::to_vec(data)
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<Vec<u8>, Self::DeserializeError> {
        serde_json::from_slice(bytes)
    }
}

impl MessageSerializer<Configuration> for JsonSerializer {
    type SerializeError = serde_json::Error;
    type DeserializeError = serde_json::Error;

    fn serialize(&self, data: &Configuration) -> Result<Vec<u8>, Self::SerializeError> {
        serde_json::to_vec(data)
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<Configuration, Self::DeserializeError> {
        serde_json::from_slice(bytes)
    }
}
