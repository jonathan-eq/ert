use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct RealForwardModelStep {
    #[serde(deserialize_with = "deserialize_status")]
    #[serde(default)]
    pub status: ForwardModelStepStatus,
    pub time: DateTime<Utc>,
    pub fm_step: String,
    #[serde(rename = "real")]
    pub real_id: String,
    #[serde(rename = "std_out")]
    pub stdout: Option<String>,
    #[serde(rename = "std_err")]
    pub stderr: Option<String>,
    pub current_memory_usage: Option<i64>,
    pub max_memory_usage: Option<i64>,
    pub cpu_seconds: Option<f64>,
    pub error: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ForwardModelStepStatus {
    Started,
    Running,
    Finished,
    Failed,
}

impl Default for ForwardModelStepStatus {
    fn default() -> Self {
        ForwardModelStepStatus::Started
    }
}

fn deserialize_status<'de, D>(deserializer: D) -> Result<ForwardModelStepStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Value = Deserialize::deserialize(deserializer)?;

    if let Some(event_type) = map.get("event_type").and_then(Value::as_str) {
        return match event_type {
            "forward_model_step.start" => Ok(ForwardModelStepStatus::Started),
            "forward_model_step.running" => Ok(ForwardModelStepStatus::Running),
            "forward_model_step.success" => Ok(ForwardModelStepStatus::Finished),
            "forward_model_step.failure" => Ok(ForwardModelStepStatus::Failed),
            _ => Err(serde::de::Error::custom("Unknown event_type")),
        };
    }

    Err(serde::de::Error::missing_field("event_type"))
}
