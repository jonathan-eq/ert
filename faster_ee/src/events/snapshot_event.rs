use serde::Serialize;

use crate::snapshots::EnsembleSnapshot;

pub const EE_SNAPSHOT_UPDATE: &str = "ee.snapshot_update";

#[derive(Serialize, Debug)]
#[serde(tag = "ee.snapshot_update")]
pub struct EESnapshotUpdateEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
#[derive(Serialize, Debug)]
#[serde(tag = "ee.snapshot")]
pub struct EEFullSnapshotEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
