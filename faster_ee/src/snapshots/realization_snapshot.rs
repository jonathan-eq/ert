use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    events::types::{FmStepId, RealId},
    update_field_if_set,
};

use super::fm_step_snapshot::FMStepSnapshot;

#[derive(Clone, Serialize)]
pub struct RealizationSnapshot {
    pub status: Option<String>,
    pub active: Option<bool>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
    pub fm_steps: Option<HashMap<FmStepId, FMStepSnapshot>>, // Might be benefitial to use None rather than empty HashMap
}

impl RealizationSnapshot {
    pub fn new() -> Self {
        RealizationSnapshot {
            status: None,
            active: None,
            start_time: None,
            end_time: None,
            exec_hosts: None,
            message: None,
            fm_steps: None,
        }
    }
    pub fn update_from_event(&mut self, event: &RealizationEvent) -> &mut Self {
        self.exec_hosts = event.get_exec_hosts();
        self.status = Some(String::from(event.get_status()));

        match event {
            RealizationEvent::RealizationRunning(inner_event) => {
                self.start_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationFailed(inner_event) => {
                self.message = inner_event.message.clone();
                self.end_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationSuccess(inner_event) => {
                self.end_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationTimeout(inner_event) => {
                self.end_time = Some(inner_event.time);
            }
            _ => {}
        }
        self
    }
    pub fn update_from(&mut self, other_snapshot: &Self) {
        update_field_if_set!(self, other_snapshot, active);
        update_field_if_set!(self, other_snapshot, end_time);
        update_field_if_set!(self, other_snapshot, exec_hosts);
        update_field_if_set!(self, other_snapshot, fm_steps); // This will probably cause a bug as it will just overwrite the hashmap rather than merge them
        update_field_if_set!(self, other_snapshot, message);
        update_field_if_set!(self, other_snapshot, start_time);
        update_field_if_set!(self, other_snapshot, status);
    }
}

pub struct RealizationPending {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

pub struct RealizationRunning {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

pub struct RealizationSuccess {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

pub struct RealizationFailed {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>, // Only used for JobState.FAILED
}

pub struct RealizationUnknown {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}
pub struct RealizationWaiting {
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
}

pub struct RealizationTimeout {
    pub real: RealId,
    pub time: DateTime<Utc>,
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
            RealizationEvent::RealizationPending(__) => "realization.pending",
            RealizationEvent::RealizationFailed(_) => "realization.failure",
            RealizationEvent::RealizationRunning(_) => "realization.running",
            RealizationEvent::RealizationSuccess(_) => "realization.success",
            RealizationEvent::RealizationUnknown(_) => "realization.unknown",
            RealizationEvent::RealizationTimeout(_) => "realization.timeout",
            RealizationEvent::RealizationWaiting(_) => "realization.waiting",
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
