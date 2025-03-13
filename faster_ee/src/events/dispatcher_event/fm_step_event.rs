use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepStart {
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    #[serde(rename = "std_out")]
    pub stdout: Option<String>,
    #[serde(rename = "std_err")]
    pub stderr: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepRunning {
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub current_memory_usage: Option<i64>,
    pub max_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepSuccess {
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    #[serde()]
    pub end_time: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct ForwardModelStepFailure {
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub end_time: NaiveDateTime,
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
