use checksum_event::ForwardModelStepChecksum;
use fm_step_event::RealForwardModelStep;
pub mod checksum_event;
use serde::{Deserialize, Serialize};
pub mod fm_step_event;

use super::ensemble_event::RealEnsembleEvent;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event_type")]
pub enum DispatcherEvent {
    #[serde(rename = "forward_model_step.start")]
    ForwardModelStepStart(RealForwardModelStep),
    #[serde(rename = "forward_model_step.running")]
    ForwardModelStepRunning(RealForwardModelStep),
    #[serde(rename = "forward_model_step.success")]
    ForwardModelStepSuccess(RealForwardModelStep),
    #[serde(rename = "forward_model_step.failure")]
    ForwardModelStepFailure(RealForwardModelStep),
    #[serde(rename = "forward_model_step.checksum")]
    ForwardModelStepChecksum(ForwardModelStepChecksum),
    #[serde(rename = "ensemble.started")]
    EnsembleStarted(RealEnsembleEvent),
    #[serde(rename = "ensemble.failed")]
    EnsembleFailed(RealEnsembleEvent),
}
