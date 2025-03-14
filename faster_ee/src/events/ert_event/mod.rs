pub mod realization_event;
pub use realization_event::*;
use serde::{Deserialize, Serialize};

use super::snapshot_event::EESnapshotUpdateEvent;

#[derive(Debug, Deserialize, Serialize)]
pub enum ErtEvent {
    #[serde(rename = "realization.waiting")]
    RealizationWaiting(RealRealization),
    #[serde(rename = "realization.pending")]
    RealizationPending(RealRealization),
    #[serde(rename = "realization.running")]
    RealizationRunning(RealRealization),
    #[serde(rename = "realization.success")]
    RealizationSuccess(RealRealization),
    #[serde(rename = "realization.failure")]
    RealizationFailed(RealRealization),
    #[serde(rename = "realization.unknown")]
    RealizationUnknown(RealRealization),
    #[serde(rename = "realization.timeout")]
    RealizationTimeout(RealRealization),
    #[serde(rename = "ee.snapshot_update")]
    EESnapshotUpdate(EESnapshotUpdateEvent),
}
