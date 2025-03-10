use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EEUserDone {
    pub monitor: String,
    pub time: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct EEUserCancel {
    pub monitor: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum ClientEvent {
    #[serde(rename = "ee.user_cancel")]
    EEUserCancel(EEUserCancel),
    #[serde(rename = "ee.user_done")]
    EEUserDone(EEUserDone),
}
