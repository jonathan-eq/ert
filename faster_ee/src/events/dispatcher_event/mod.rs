use checksum_event::ForwardModelStepChecksum;
use fm_step_event::{
    ForwardModelStepFailure, ForwardModelStepRunning, ForwardModelStepStart,
    ForwardModelStepSuccess,
};
pub mod checksum_event;
use serde::Deserialize;
pub mod fm_step_event;
pub use fm_step_event::FMEvent;

use crate::snapshots::realization_snapshot::RealizationWaiting;

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum DispatcherEvent {
    #[serde(rename = "forward_model_step.start")]
    ForwardModelStepStart(ForwardModelStepStart),
    #[serde(rename = "forward_model_step.running")]
    ForwardModelStepRunning(ForwardModelStepRunning),
    #[serde(rename = "forward_model_step.success")]
    ForwardModelStepSuccess(ForwardModelStepSuccess),
    #[serde(rename = "forward_model_step.failure")]
    ForwardModelStepFailure(ForwardModelStepFailure),
    #[serde(rename = "realization.waiting")]
    RealizationWaiting(RealizationWaiting),
    #[serde(rename = "forward_model_step.checksum")]
    Checksum(ForwardModelStepChecksum),
}
