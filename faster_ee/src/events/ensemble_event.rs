use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleStarted {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    pub time: NaiveDateTime,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleSucceeded {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    pub time: NaiveDateTime,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleFailed {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    pub time: NaiveDateTime,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleCancelled {
    #[serde(rename = "ensemble")]
    pub ensemble_id: String,
    pub time: NaiveDateTime,
}

pub enum EnsembleEvent {
    EnsembleStarted(EnsembleStarted),
    EnsembleSucceeded(EnsembleSucceeded),
    EnsembleFailed(EnsembleFailed),
    EnsembleCancelled(EnsembleCancelled),
}

impl EnsembleEvent {
    pub fn get_status(&self) -> &'static str {
        match self {
            EnsembleEvent::EnsembleStarted(_) => "Starting",
            EnsembleEvent::EnsembleSucceeded(_) => "Stopped",
            EnsembleEvent::EnsembleFailed(_) => "Cancelled",
            EnsembleEvent::EnsembleCancelled(_) => "Failed",
        }
    }
}
