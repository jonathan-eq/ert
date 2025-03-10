use crate::{events::dispatcher_event::FMEvent, update_field_if_set};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Status {
    const STATUS: &'static str;
}
#[derive(Clone, Serialize, Debug)]
pub struct FMStepSnapshot {
    pub status: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub index: Option<String>,
    pub current_memory_usage: Option<i64>,
    pub max_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub stdout: Option<String>,
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
        return self;
    }
}
