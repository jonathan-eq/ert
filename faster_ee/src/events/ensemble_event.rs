use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleStarted {
    pub time: NaiveDateTime,
    pub ensemble: String,
    #[serde(default = "EnsembleStatus::get_started")]
    pub status: EnsembleStatus,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleSucceeded {
    pub time: NaiveDateTime,
    pub ensemble: String,
    #[serde(default = "EnsembleStatus::get_succeeded")]
    pub status: EnsembleStatus,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleFailed {
    pub time: NaiveDateTime,
    pub ensemble: String,
    #[serde(default = "EnsembleStatus::get_failed")]
    pub status: EnsembleStatus,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnsembleCancelled {
    pub time: NaiveDateTime,
    pub ensemble: String,
    #[serde(default = "EnsembleStatus::get_cancelled")]
    pub status: EnsembleStatus,
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
    pub fn get_started() -> Self {
        EnsembleStatus::Started
    }
    pub fn get_succeeded() -> Self {
        EnsembleStatus::Succeeded
    }
    pub fn get_failed() -> Self {
        EnsembleStatus::Failed
    }
    pub fn get_cancelled() -> Self {
        EnsembleStatus::Cancelled
    }
}

pub enum EnsembleEvent {
    Started(EnsembleStarted),
    Succeeded(EnsembleSucceeded),
    Failed(EnsembleFailed),
    Cancelled(EnsembleCancelled),
}

impl EnsembleEvent {
    pub fn get_ensemble_id(&self) -> String {
        match self {
            EnsembleEvent::Started(event) => event.ensemble.clone(),
            EnsembleEvent::Succeeded(event) => event.ensemble.clone(),
            EnsembleEvent::Failed(event) => event.ensemble.clone(),
            EnsembleEvent::Cancelled(event) => event.ensemble.clone(),
        }
    }
    pub fn get_status(&self) -> EnsembleStatus {
        match self {
            EnsembleEvent::Started(event) => event.status.clone(),
            EnsembleEvent::Succeeded(event) => event.status.clone(),
            EnsembleEvent::Failed(event) => event.status.clone(),
            EnsembleEvent::Cancelled(event) => event.status.clone(),
        }
    }
}
