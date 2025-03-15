use dispatcher_event::fm_step_event::RealForwardModelStep;
use ensemble_event::RealEnsembleEvent;
use serde::Serialize;
use snapshot_event::EESnapshotEvent;

use ert_event::RealRealization;
pub mod client_event;
pub mod dispatcher_event;
pub mod ert_event;

pub mod ensemble_event;
pub mod snapshot_event;
pub mod types;
pub enum Event {
    EnsembleEvent(RealEnsembleEvent),
    FMEvent(RealForwardModelStep),
    RealizationEvent(RealRealization),
    EESnapshotUpdateEvent(EESnapshotEvent),
}

#[derive(Serialize, Debug)]
pub struct EECancelled {
    pub ensemble_id: String,
    pub monitor: Option<String>,
}
