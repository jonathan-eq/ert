use crate::update_field_if_set;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub trait Status {
    const STATUS: &'static str;
}
#[derive(Clone, Serialize)]
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
    pub fn update_from(&mut self, other_snapshot: Self) {
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
}

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepStart {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}
impl Status for ForwardModelStepStart {
    const STATUS: &'static str = "Pending";
}
#[derive(Debug, Deserialize)]
pub struct ForwardModelStepRunning {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub current_memory_usage: Option<i64>,
    pub max_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
}
impl Status for ForwardModelStepRunning {
    const STATUS: &'static str = "Running";
}
#[derive(Debug, Deserialize)]
pub struct ForwardModelStepSuccess {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub end_time: DateTime<Utc>,
}
impl Status for ForwardModelStepSuccess {
    const STATUS: &'static str = "Finished";
}
#[derive(Debug, Deserialize)]
pub struct ForwardModelStepFailure {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub end_time: DateTime<Utc>,
    pub error: Option<String>,
}
impl Status for ForwardModelStepFailure {
    const STATUS: &'static str = "Failed";
}
pub enum FMEvent {
    ForwardModelStepStart(ForwardModelStepStart),
    ForwardModelStepRunning(ForwardModelStepRunning),
    ForwardModelStepSuccess(ForwardModelStepSuccess),
    ForwardModelStepFailure(ForwardModelStepFailure),
}

impl FMEvent {
    // Accessing the STATUS constant from each variant
    pub fn get_status(&self) -> &'static str {
        match self {
            FMEvent::ForwardModelStepStart(_) => ForwardModelStepStart::STATUS,
            FMEvent::ForwardModelStepRunning(_) => ForwardModelStepRunning::STATUS,
            FMEvent::ForwardModelStepSuccess(_) => ForwardModelStepSuccess::STATUS,
            FMEvent::ForwardModelStepFailure(_) => ForwardModelStepFailure::STATUS,
        }
    }
    pub fn get_fm_step_id(&self) -> String {
        // We should not clone here, but instead transfer ownership. We won't be needing the events after this anyways.
        match self {
            FMEvent::ForwardModelStepStart(inner) => inner.fm_step.clone(),
            FMEvent::ForwardModelStepRunning(inner) => inner.fm_step.clone(),
            FMEvent::ForwardModelStepSuccess(inner) => inner.fm_step.clone(),
            FMEvent::ForwardModelStepFailure(inner) => inner.fm_step.clone(),
        }
    }
    pub fn get_real_id(&self) -> String {
        // We should not clone here, but instead transfer ownership. We won't be needing the events after this anyways.
        match self {
            FMEvent::ForwardModelStepStart(inner) => inner.real_id.clone(),
            FMEvent::ForwardModelStepRunning(inner) => inner.real_id.clone(),
            FMEvent::ForwardModelStepSuccess(inner) => inner.real_id.clone(),
            FMEvent::ForwardModelStepFailure(inner) => inner.real_id.clone(),
        }
    }
}
