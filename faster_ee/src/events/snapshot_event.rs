use serde::{Deserialize, Serialize};

use crate::snapshots::EnsembleSnapshot;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct EESnapshotEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}
impl EESnapshotEvent {
    pub fn new(snapshot: EnsembleSnapshot, ensemble: String) -> Self {
        EESnapshotEvent {
            snapshot: snapshot.clone(),
            ensemble: ensemble.clone(),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
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

#[derive(Serialize, Debug, Deserialize)]
pub struct EEFullSnapshotEvent {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}

impl EEFullSnapshotEvent {
    pub fn new(snapshot: EnsembleSnapshot, ensemble: String) -> Self {
        EEFullSnapshotEvent { snapshot, ensemble }
    }
}
