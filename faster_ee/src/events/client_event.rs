use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EECancelled;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EEUserEvent {
    pub monitor: String,
    pub time: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event_type")]
pub enum ClientEvent {
    #[serde(rename = "ee.user_cancel")]
    EEUserCancel(EECancelled),
    #[serde(rename = "ee.user_done")]
    EEUserDone(EEUserEvent),
}
