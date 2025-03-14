use dispatcher_event::fm_step_event::RealForwardModelStep;
use ensemble_event::RealEnsembleEvent;
use snapshot_event::EESnapshotUpdateEvent;

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
    EESnapshotUpdateEvent(EESnapshotUpdateEvent),
}
