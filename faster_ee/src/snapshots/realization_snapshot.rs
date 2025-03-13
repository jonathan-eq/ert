use std::collections::HashMap;

use crate::events::ert_event::RealizationEvent;
use crate::utils::is_none_or_empty;
use crate::{events::types::FmStepId, update_field_if_not_empty, update_field_if_set};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::fm_step_snapshot::FMStepSnapshot;

#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
pub struct RealizationSnapshot {
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<chrono::NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<chrono::NaiveDateTime>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub exec_hosts: Option<String>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(rename = "fm_steps")]
    pub fm_steps: HashMap<FmStepId, FMStepSnapshot>, // Might be benefitial to use None rather than empty HashMap
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
            fm_steps: HashMap::new(),
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
        update_field_if_not_empty!(self, other_snapshot, fm_steps);
        update_field_if_set!(self, other_snapshot, message);
        update_field_if_set!(self, other_snapshot, start_time);
        update_field_if_set!(self, other_snapshot, status);
    }
}
