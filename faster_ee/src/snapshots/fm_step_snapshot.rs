use crate::events::dispatcher_event::fm_step_event::{
    ForwardModelStepEvent, ForwardModelStepStatus, RealForwardModelStep,
};
use crate::utils::is_none_or_empty;
use crate::{update_field, update_field_if_set};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
pub struct FMStepSnapshot {
    //#[serde(deserialize_with = "deserialize_status")]
    pub status: Option<ForwardModelStepStatus>,
    pub start_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_memory_usage: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_usage: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_seconds: Option<f64>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub stderr: Option<String>,
}
impl Default for FMStepSnapshot {
    fn default() -> Self {
        FMStepSnapshot::new()
    }
}
impl FMStepSnapshot {
    pub fn new() -> Self {
        Self {
            status: None,
            cpu_seconds: None,
            current_memory_usage: None,
            max_memory_usage: None,
            end_time: None,
            error: None,
            start_time: None,
            index: None,
            name: None,
            stderr: None,
            stdout: None,
        }
    }
    pub fn update_from(&mut self, other_snapshot: &Self) {
        update_field!(self, other_snapshot, status);
        update_field_if_set!(self, other_snapshot, index);
        update_field_if_set!(self, other_snapshot, cpu_seconds);
        update_field_if_set!(self, other_snapshot, current_memory_usage);
        update_field_if_set!(self, other_snapshot, max_memory_usage);
        update_field_if_set!(self, other_snapshot, end_time);
        update_field_if_set!(self, other_snapshot, error);
        update_field_if_set!(self, other_snapshot, start_time);
        update_field_if_set!(self, other_snapshot, name);
        update_field_if_set!(self, other_snapshot, stderr);
        update_field_if_set!(self, other_snapshot, stdout);
    }
    pub fn update_from_event(&mut self, event: &ForwardModelStepEvent) -> &mut Self {
        self.index = Some(event.get_fm_step_id().clone());
        self.status = Some(event.get_status().clone());
        match event {
            ForwardModelStepEvent::Start(event) => {
                self.start_time = Some(event.time);
                self.stdout = event.stdout.clone();
                self.stderr = event.stderr.clone();
            }
            ForwardModelStepEvent::Running(event) => {
                self.current_memory_usage = event.current_memory_usage;
                self.max_memory_usage = event.max_memory_usage;
                self.cpu_seconds = event.cpu_seconds;
            }
            ForwardModelStepEvent::Success(event) => {
                self.end_time = Some(event.time);
            }
            ForwardModelStepEvent::Failure(event) => {
                self.end_time = Some(event.time);
                self.error = Some(event.error_msg.clone());
            }
        }
        return self;
    }
}
