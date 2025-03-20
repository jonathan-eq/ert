use chrono::NaiveDateTime;
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
