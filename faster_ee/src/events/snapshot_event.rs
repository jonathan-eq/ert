use serde::Serialize;

use crate::snapshots::EnsembleSnapshot;

pub const EE_SNAPSHOT_UPDATE: &str = "ee.snapshot_update";

#[derive(Serialize)]
#[serde(tag = "ee.snapshot")]
pub struct EESnapshotUpdateEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
