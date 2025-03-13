use serde::{Deserialize, Serialize};

use crate::snapshots::EnsembleSnapshot;

#[derive(Serialize, Debug, Deserialize)]
pub struct EESnapshotUpdateEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
impl EESnapshotUpdateEvent {
    pub fn new(snapshot: EnsembleSnapshot, ensemble: String) -> Self {
        EESnapshotUpdateEvent {
            snapshot: snapshot.clone(),
            ensemble: ensemble.clone(),
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
