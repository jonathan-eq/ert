use dispatcher_event::FMEvent;
use ensemble_event::EnsembleEvent;
use snapshot_event::EESnapshotUpdateEvent;

use ert_event::RealizationEvent;
pub mod client_event;
pub mod dispatcher_event;
pub mod ert_event;

pub mod ensemble_event;
pub mod snapshot_event;
pub mod types;
pub enum Event {
    EnsembleEvent(EnsembleEvent),
    FMEvent(FMEvent),
    RealizationEvent(RealizationEvent),
    EESnapshotUpdateEvent(EESnapshotUpdateEvent),
}
