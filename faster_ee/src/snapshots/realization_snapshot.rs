use std::collections::HashMap;

use crate::events::ert_event::realization_event::deserialize_status;
use crate::events::ert_event::RealRealization;
use crate::update_field;
use crate::utils::is_none_or_empty;
use crate::{events::types::FmStepId, update_field_if_not_empty, update_field_if_set};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use super::fm_step_snapshot::FMStepSnapshot;

#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
pub struct RealizationSnapshot {
    //#[serde(deserialize_with = "deserialize_status")]
    #[serde(default)]
    pub status: Option<RealizationState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub exec_hosts: Option<String>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(rename = "fm_steps")]
    pub fm_steps: HashMap<FmStepId, FMStepSnapshot>, // Might be benefitial to use None rather than empty HashMap
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RealizationState {
    Waiting,
    Pending,
    Running,
    Failed,
    Finished,
    Unknown,
    Timeout,
}

impl RealizationState {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Waiting => "Waiting",
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Failed => "Failed",
            Self::Finished => "Finished",
            Self::Unknown => "Unknown",
            Self::Timeout => "Timeout",
        }
    }
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
    pub fn update_from_event(&mut self, event: &RealRealization) -> &mut Self {
        self.exec_hosts = event.exec_hosts.clone();
        if let Some(new_status) = event.status.clone() {
            self.status = Some(new_status);
        }
        if let Some(status) = &self.status {
            match status {
                RealizationState::Running => {
                    self.start_time = Some(event.time);
                }
                RealizationState::Failed => {
                    self.message = event.message.clone();
                    self.end_time = Some(event.time);
                }
                RealizationState::Finished => {
                    self.end_time = Some(event.time);
                }
                RealizationState::Timeout => {
                    self.end_time = Some(event.time);
                }
                _ => {}
            }
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
        update_field!(self, other_snapshot, status);
    }
}
