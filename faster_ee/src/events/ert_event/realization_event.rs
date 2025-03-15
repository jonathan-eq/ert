use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{events::types::RealId, snapshots::realization_snapshot::RealizationState};

#[derive(Debug, Deserialize, Serialize)]
pub struct RealRealization {
    pub status: RealizationState,
    pub real: RealId,
    pub time: DateTime<Utc>,
    pub ensemble: Option<String>,
    pub queue_event_type: Option<String>,
    pub exec_hosts: Option<String>,
    pub message: Option<String>,
}
