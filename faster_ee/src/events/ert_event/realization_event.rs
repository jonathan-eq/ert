use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

use crate::events::types::RealId;

#[derive(Debug, Deserialize)]
pub struct RealizationPending {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RealizationRunning {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RealizationSuccess {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RealizationFailed {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>, // Only used for JobState.FAILED
}

#[derive(Debug, Deserialize)]
pub struct RealizationUnknown {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct RealizationWaiting {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RealizationTimeout {
    pub real: RealId,
    pub time: NaiveDateTime,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

pub enum RealizationEvent {
    RealizationPending(RealizationPending),
    RealizationRunning(RealizationRunning),
    RealizationSuccess(RealizationSuccess),
    RealizationFailed(RealizationFailed),
    RealizationUnknown(RealizationUnknown),
    RealizationWaiting(RealizationWaiting),
    RealizationTimeout(RealizationTimeout),
}

impl RealizationEvent {
    pub fn get_status(&self) -> &'static str {
        match self {
            RealizationEvent::RealizationPending(__) => "Pending",
            RealizationEvent::RealizationFailed(_) => "Failed",
            RealizationEvent::RealizationRunning(_) => "Running",
            RealizationEvent::RealizationSuccess(_) => "Success",
            RealizationEvent::RealizationUnknown(_) => "Unknown",
            RealizationEvent::RealizationTimeout(_) => "Timeout",
            RealizationEvent::RealizationWaiting(_) => "Waiting",
        }
    }
    pub fn get_exec_hosts(&self) -> Option<String> {
        match self {
            RealizationEvent::RealizationPending(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationFailed(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationRunning(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationSuccess(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationUnknown(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationTimeout(inner) => inner.exec_hosts.clone(),
            RealizationEvent::RealizationWaiting(inner) => inner.exec_hosts.clone(),
        }
    }
    pub fn get_real_id(&self) -> RealId {
        match self {
            RealizationEvent::RealizationPending(inner) => inner.real.clone(),
            RealizationEvent::RealizationFailed(inner) => inner.real.clone(),
            RealizationEvent::RealizationRunning(inner) => inner.real.clone(),
            RealizationEvent::RealizationSuccess(inner) => inner.real.clone(),
            RealizationEvent::RealizationUnknown(inner) => inner.real.clone(),
            RealizationEvent::RealizationTimeout(inner) => inner.real.clone(),
            RealizationEvent::RealizationWaiting(inner) => inner.real.clone(),
        }
    }
}
