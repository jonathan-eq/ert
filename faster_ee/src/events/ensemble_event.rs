use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct RealEnsembleEvent {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_status")]
    pub state: EnsembleStatus,
    pub time: NaiveDateTime,
}
impl Default for EnsembleStatus {
    fn default() -> Self {
        EnsembleStatus::Started
    }
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<EnsembleStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Value = Deserialize::deserialize(deserializer)?;

    if let Some(event_type) = map.get("event_type").and_then(Value::as_str) {
        return match event_type {
            "ensemble.started" => Ok(EnsembleStatus::Started),
            "ensemble.cancelled" => Ok(EnsembleStatus::Cancelled),
            "ensemble.success" => Ok(EnsembleStatus::Succeeded),
            "ensemble.failure" => Ok(EnsembleStatus::Failed),
            _ => Err(serde::de::Error::custom("Unknown event_type")),
        };
    }

    Err(serde::de::Error::missing_field("event_type"))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EnsembleStatus {
    Started,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}

impl EnsembleStatus {
    pub fn get_status(&self) -> &'static str {
        match self {
            EnsembleStatus::Started => "Starting",
            EnsembleStatus::Succeeded => "Stopped",
            EnsembleStatus::Failed => "Cancelled",
            EnsembleStatus::Cancelled => "Failed",
            EnsembleStatus::Unknown => "Unknown",
        }
    }
}
