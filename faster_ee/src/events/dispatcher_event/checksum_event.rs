use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct _Checksum {
    #[serde(rename = "type")]
    pub _type: String,
    pub path: String,
    pub md5sum: Option<String>,
    pub error: Option<String>,
    pub time: Option<NaiveDateTime>,
    pub error_message: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ForwardModelStepChecksum {
    time: NaiveDateTime,
    ensemble: Option<String>,
    real: String,
    #[serde(default)]
    checksums: HashMap<String, HashMap<String, _Checksum>>,
}
