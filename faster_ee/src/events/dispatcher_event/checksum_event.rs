use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
struct _Checksum {
    #[serde(rename = "type")]
    pub _type: String,
    pub path: String,
    pub md5sum: Option<String>,
    pub error: Option<String>,
    pub time: DateTime<Utc>,
    pub error_message: Option<String>,
}
#[derive(Deserialize, Debug, Serialize)]
struct Checksum {
    pub data: HashMap<String, _Checksum>,
    pub run_path: String,
}
#[derive(Deserialize, Debug, Serialize)]
pub struct ForwardModelStepChecksum {
    ensemble: Option<String>,
    real: String,
    #[serde(default)]
    checksums: HashMap<String, Checksum>,
}
