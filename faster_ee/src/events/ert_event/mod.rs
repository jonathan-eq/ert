pub mod realization_event;
use chrono::NaiveDateTime;
pub use realization_event::*;
use serde::{Deserialize, Serialize};

use super::snapshot_event::EESnapshotUpdateEvent;

#[derive(Debug, Deserialize, Serialize)]
pub enum ErtEvent {
    #[serde(rename = "realization.waiting")]
    RealizationWaiting(RealizationWaiting),
    #[serde(rename = "realization.pending")]
    RealizationPending(RealizationPending),
    #[serde(rename = "realization.running")]
    RealizationRunning(RealizationRunning),
    #[serde(rename = "realization.success")]
    RealizationSuccess(RealizationSuccess),
    #[serde(rename = "realization.failure")]
    RealizationFailed(RealizationFailed),
    #[serde(rename = "realization.unknown")]
    RealizationUnknown(RealizationUnknown),
    #[serde(rename = "realization.timeout")]
    RealizationTimeout(RealizationTimeout),
    #[serde(rename = "ee.snapshot_update")]
    EESnapshotUpdate(EESnapshotUpdateEvent),
}
