use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RealForwardModelStep {
    pub status: Option<ForwardModelStepStatus>,
    pub time: NaiveDateTime,
    pub fm_step: String,
    pub real_id: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub current_memory_usage: Option<i64>,
    pub max_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardModelStepStart {
    #[serde(default = "ForwardModelStepStatus::get_starting")]
    pub status: ForwardModelStepStatus,
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub ensemble: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardModelStepRunning {
    #[serde(default = "ForwardModelStepStatus::get_running")]
    pub status: ForwardModelStepStatus,
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub ensemble: Option<String>,
    pub max_memory_usage: Option<i64>,
    pub current_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardModelStepSuccess {
    #[serde(default = "ForwardModelStepStatus::get_finished")]
    pub status: ForwardModelStepStatus,
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub ensemble: Option<String>,
    pub current_memory_usage: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForwardModelStepFailure {
    #[serde(default = "ForwardModelStepStatus::get_failed")]
    pub status: ForwardModelStepStatus,
    pub time: NaiveDateTime,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    pub ensemble: Option<String>,
    pub error_msg: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ForwardModelStepStatus {
    Pending,
    Finished,
    Failed,
    Running,
}

impl ForwardModelStepStatus {
    fn get_starting() -> ForwardModelStepStatus {
        ForwardModelStepStatus::Pending
    }
    fn get_finished() -> ForwardModelStepStatus {
        ForwardModelStepStatus::Finished
    }
    fn get_failed() -> ForwardModelStepStatus {
        ForwardModelStepStatus::Failed
    }
    fn get_running() -> ForwardModelStepStatus {
        ForwardModelStepStatus::Running
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ForwardModelStepEvent {
    Start(ForwardModelStepStart),
    Running(ForwardModelStepRunning),
    Success(ForwardModelStepSuccess),
    Failure(ForwardModelStepFailure),
}

impl ForwardModelStepEvent {
    pub fn get_real_id(&self) -> String {
        match self {
            ForwardModelStepEvent::Start(event) => event.real_id.clone(),
            ForwardModelStepEvent::Running(event) => event.real_id.clone(),
            ForwardModelStepEvent::Success(event) => event.real_id.clone(),
            ForwardModelStepEvent::Failure(event) => event.real_id.clone(),
        }
    }
    pub fn get_status(&self) -> ForwardModelStepStatus {
        match self {
            ForwardModelStepEvent::Start(event) => event.status.clone(),
            ForwardModelStepEvent::Running(event) => event.status.clone(),
            ForwardModelStepEvent::Success(event) => event.status.clone(),
            ForwardModelStepEvent::Failure(event) => event.status.clone(),
        }
    }
    pub fn get_fm_step_id(&self) -> String {
        match self {
            ForwardModelStepEvent::Start(event) => event.fm_step.clone(),
            ForwardModelStepEvent::Running(event) => event.fm_step.clone(),
            ForwardModelStepEvent::Success(event) => event.fm_step.clone(),
            ForwardModelStepEvent::Failure(event) => event.fm_step.clone(),
        }
    }
}
