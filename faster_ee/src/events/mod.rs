use dispatcher_event::fm_step_event::ForwardModelStepEvent;
use ensemble_event::EnsembleEvent;
use serde::{Deserialize, Serialize};
use snapshot_event::EESnapshotEvent;

use ert_event::RealizationEvent;
pub mod client_event;
pub mod dispatcher_event;
pub mod ert_event;

pub mod ensemble_event;
pub mod snapshot_event;
pub mod types;
pub enum Event {
    EnsembleEvent(EnsembleEvent),
    FMEvent(ForwardModelStepEvent),
    RealizationEvent(RealizationEvent),
    EESnapshotUpdateEvent(EESnapshotEvent),
    EEFullSnapshotEvent(EESnapshotEvent),
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct EECancelled {
    pub ensemble_id: String,
    pub monitor: Option<String>,
}
