use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{events::types::RealId, snapshots::realization_snapshot::RealizationState};
#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationPending {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_pending")]
    pub status: RealizationState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationWaiting {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_waiting")]
    pub status: RealizationState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationRunning {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_running")]
    pub status: RealizationState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationFinished {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_finished")]
    pub status: RealizationState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationFailed {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_failed")]
    pub status: RealizationState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationUnknown {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_unknown")]
    pub status: RealizationState,
}

impl RealizationUnknown {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealizationTimeout {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    #[serde(default = "RealizationState::get_timeout")]
    pub status: RealizationState,
}

impl RealizationTimeout {}
pub enum RealizationEvent {
    RealizationPending(RealizationPending),
    RealizationRunning(RealizationRunning),
    RealizationSuccess(RealizationFinished),
    RealizationFailed(RealizationFailed),
    RealizationUnknown(RealizationUnknown),
    RealizationWaiting(RealizationWaiting),
    RealizationTimeout(RealizationTimeout),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RealRealization {
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(default)]
    pub status: Option<RealizationState>,
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
}

pub fn deserialize_status<'de, D>(deserializer: D) -> Result<Option<RealizationState>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Value = Deserialize::deserialize(deserializer)?;

    if let Some(event_type) = map.get("event_type").and_then(Value::as_str) {
        return match event_type {
            "realization.waiting" => Ok(Some(RealizationState::Waiting)),
            "realization.pending" => Ok(Some(RealizationState::Pending)),
            "realization.running" => Ok(Some(RealizationState::Running)),
            "realization.success" => Ok(Some(RealizationState::Finished)),
            "realization.failure" => Ok(Some(RealizationState::Failed)),
            "realization.unknown" => Ok(Some(RealizationState::Unknown)),
            "realization.timeout" => Ok(Some(RealizationState::Timeout)),
            _ => Err(serde::de::Error::custom("Unknown event_type")),
        };
    }

    Err(serde::de::Error::missing_field("event_type"))
}
