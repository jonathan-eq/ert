use checksum_event::ForwardModelStepChecksum;
use fm_step_event::{
    ForwardModelStepFailure, ForwardModelStepRunning, ForwardModelStepStart,
    ForwardModelStepSuccess,
};
pub mod checksum_event;
use serde::Deserialize;
pub mod fm_step_event;
pub use fm_step_event::FMEvent;

use super::{
    ensemble_event::{EnsembleFailed, EnsembleStarted},
    ert_event::realization_event::RealizationPending,
};

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
    #[serde(rename = "forward_model_step.checksum")]
    ForwardModelStepChecksum(ForwardModelStepChecksum),
    #[serde(rename = "ensemble.started")]
    EnsembleStarted(EnsembleStarted),
    #[serde(rename = "ensemble.failed")]
    EnsembleFailed(EnsembleFailed),
}
