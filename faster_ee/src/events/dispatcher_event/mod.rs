use fm_step_event::{
    ForwardModelStepFailure, ForwardModelStepRunning, ForwardModelStepStart,
    ForwardModelStepSuccess,
};
use serde::Deserialize;
pub mod fm_step_event;
pub use fm_step_event::FMEvent;

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum DispatcherEvent {
    #[serde(rename = "forward_model.start")]
    ForwardModelStepStart(ForwardModelStepStart),
    #[serde(rename = "forward_model.running")]
    ForwardModelStepRunning(ForwardModelStepRunning),
    #[serde(rename = "forward_model.success")]
    ForwardModelStepSuccess(ForwardModelStepSuccess),
    #[serde(rename = "forward_model.failure")]
    ForwardModelStepFailure(ForwardModelStepFailure),
}
