use serde::Serialize;

use crate::snapshots::EnsembleSnapshot;

#[derive(Serialize, Debug)]
pub struct EESnapshotUpdateEvent {
    pub event_type: &'static str,
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
impl EESnapshotUpdateEvent {
    pub fn new(snapshot: EnsembleSnapshot, ensemble: String) -> Self {
        EESnapshotUpdateEvent {
            event_type: "ee.snapshot_update",
            snapshot,
            ensemble,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EEFullSnapshotEvent {
    pub event_type: &'static str,
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}

impl EEFullSnapshotEvent {
    pub fn new(snapshot: EnsembleSnapshot, ensemble: String) -> Self {
        EEFullSnapshotEvent {
            event_type: "ee.snapshot",
            snapshot,
            ensemble,
        }
    }
}
