use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RealEnsembleEvent {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    pub state: EnsembleStatus,
    pub time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EnsembleStatus {
    Started,
    Succeeded,
    Failed,
    Cancelled,
}

impl EnsembleStatus {
    pub fn get_status(&self) -> &'static str {
        match self {
            EnsembleStatus::Started => "Starting",
            EnsembleStatus::Succeeded => "Stopped",
            EnsembleStatus::Failed => "Cancelled",
            EnsembleStatus::Cancelled => "Failed",
        }
    }
}
