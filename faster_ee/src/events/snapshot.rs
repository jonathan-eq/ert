use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::events::types::Id;

use super::snapshots::ensemble_snapshot::EnsembleSnapshot;

pub const EE_SNAPSHOT_UPDATE: &str = "ee.snapshot_update";

#[derive(Serialize)]
pub struct EESnapshotUpdate {
    pub snapshot: EnsembleSnapshot,
    pub ensemble: String,
}

pub fn event_to_json(input: &str) -> Vec<u8> {
    // THIS WONT WORK NOW!
    return input.as_bytes().to_vec();
}
