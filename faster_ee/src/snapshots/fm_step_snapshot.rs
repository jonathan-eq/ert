use crate::utils::is_none_or_empty;
use crate::{events::dispatcher_event::FMEvent, update_field_if_set};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct FMStepSnapshot {
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub status: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
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
        update_field_if_set!(self, other_snapshot, status);
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
    pub fn update_from_event(&mut self, event: &FMEvent) -> &mut Self {
        self.status = Some(String::from(event.get_status()));
        self.index = Some(event.get_fm_step_id().clone());
        match event {
            FMEvent::ForwardModelStepStart(inner_event) => {
                self.start_time = Some(inner_event.time);
                self.stdout = inner_event.stdout.clone();
                self.stderr = inner_event.stderr.clone();
            }
            FMEvent::ForwardModelStepRunning(inner_event) => {
                self.current_memory_usage = inner_event.current_memory_usage;
                self.max_memory_usage = inner_event.max_memory_usage;
                self.cpu_seconds = inner_event.cpu_seconds;
            }
            FMEvent::ForwardModelStepSuccess(event) => {
                self.end_time = Some(event.end_time);
                self.error = Some(String::new());
            }
            FMEvent::ForwardModelStepFailure(event) => {
                self.end_time = Some(event.end_time);
                self.error = event.error.clone();
            }
        }
        println!("CONSTRUCTED FM STEP SNAPSHOT: {:?}", self);
        return self;
    }
}
