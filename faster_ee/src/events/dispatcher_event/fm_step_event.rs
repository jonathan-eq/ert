use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ForwardModelStepStatus {
    Running,
    Finished,
    Failed,
    Pending,
}

impl<'de> Deserialize<'de> for RealForwardModelStep {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStep {
            event_type: String,
            time: NaiveDateTime,
            fm_step: String,
            real: String,
            std_out: Option<String>,
            std_err: Option<String>,
            current_memory_usage: Option<i64>,
            max_memory_usage: Option<i64>,
            cpu_seconds: Option<f64>,
            error: Option<String>,
        }

        let raw = RawStep::deserialize(deserializer)?;
        let status = match raw.event_type.as_str() {
            "forward_model_step.running" => Some(ForwardModelStepStatus::Running),
            "forward_model_step.finished" => Some(ForwardModelStepStatus::Finished),
            "forward_model_step.failed" => Some(ForwardModelStepStatus::Failed),
            "forward_model_step.pending" => Some(ForwardModelStepStatus::Pending),
            _ => None,
        };

        Ok(RealForwardModelStep {
            status,
            time: raw.time,
            fm_step: raw.fm_step,
            real_id: raw.real,
            stdout: raw.std_out,
            stderr: raw.std_err,
            current_memory_usage: raw.current_memory_usage,
            max_memory_usage: raw.max_memory_usage,
            cpu_seconds: raw.cpu_seconds,
            error: raw.error,
        })
    }
}
