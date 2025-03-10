use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepStart {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepSuccess {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepFailure {
    pub time: DateTime<Utc>,
    pub fm_step: String,
    pub real_id: String,
    pub end_time: DateTime<Utc>,
    pub error: Option<String>,
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
            FMEvent::ForwardModelStepStart(_) => "Pending",
            FMEvent::ForwardModelStepRunning(_) => "Running",
            FMEvent::ForwardModelStepSuccess(_) => "Finished",
            FMEvent::ForwardModelStepFailure(_) => "Failed",
        }
    }
    pub fn get_fm_step_id(&self) -> &String {
        match self {
            FMEvent::ForwardModelStepStart(inner) => &inner.fm_step,
            FMEvent::ForwardModelStepRunning(inner) => &inner.fm_step,
            FMEvent::ForwardModelStepSuccess(inner) => &inner.fm_step,
            FMEvent::ForwardModelStepFailure(inner) => &inner.fm_step,
        }
    }
    pub fn get_real_id(&self) -> &String {
        match self {
            FMEvent::ForwardModelStepStart(inner) => &inner.real_id,
            FMEvent::ForwardModelStepRunning(inner) => &inner.real_id,
            FMEvent::ForwardModelStepSuccess(inner) => &inner.real_id,
            FMEvent::ForwardModelStepFailure(inner) => &inner.real_id,
        }
    }
}
