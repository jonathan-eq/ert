use std::time::Duration;

pub struct Id;
impl Id {
    // Forward Model Step Constants
    pub const FORWARD_MODEL_STEP_START: &'static str = "forward_model_step.start";
    pub const FORWARD_MODEL_STEP_RUNNING: &'static str = "forward_model_step.running";
    pub const FORWARD_MODEL_STEP_SUCCESS: &'static str = "forward_model_step.success";
    pub const FORWARD_MODEL_STEP_FAILURE: &'static str = "forward_model_step.failure";
    pub const FORWARD_MODEL_STEP_CHECKSUM: &'static str = "forward_model_step.checksum";

    // Realization Constants
    pub const REALIZATION_FAILURE: &'static str = "realization.failure";
    pub const REALIZATION_PENDING: &'static str = "realization.pending";
    pub const REALIZATION_RUNNING: &'static str = "realization.running";
    pub const REALIZATION_SUCCESS: &'static str = "realization.success";
    pub const REALIZATION_UNKNOWN: &'static str = "realization.unknown";
    pub const REALIZATION_WAITING: &'static str = "realization.waiting";
    pub const REALIZATION_TIMEOUT: &'static str = "realization.timeout";

    // Ensemble Constants
    pub const ENSEMBLE_STARTED: &'static str = "ensemble.started";
    pub const ENSEMBLE_SUCCEEDED: &'static str = "ensemble.succeeded";
    pub const ENSEMBLE_CANCELLED: &'static str = "ensemble.cancelled";
    pub const ENSEMBLE_FAILED: &'static str = "ensemble.failed";

    // EE Constants
    pub const EE_SNAPSHOT: &'static str = "ee.snapshot";
    pub const EE_SNAPSHOT_UPDATE: &'static str = "ee.snapshot_update";
    pub const EE_TERMINATED: &'static str = "ee.terminated";
    pub const EE_USER_CANCEL: &'static str = "ee.user_cancel";
    pub const EE_USER_DONE: &'static str = "ee.user_done";

    // Ensemble Types Collection
    pub const ENSEMBLE_TYPES: [&'static str; 4] = [
        Self::ENSEMBLE_STARTED,
        Self::ENSEMBLE_FAILED,
        Self::ENSEMBLE_SUCCEEDED,
        Self::ENSEMBLE_CANCELLED,
    ];
}

pub const DISCONNECT_MSG: &str = "DISCONNECT";
pub struct HeartBeat {
    pub msg: String,
}
impl HeartBeat {
    pub fn new() -> Self {
        HeartBeat {
            msg: String::from("HEARTBEAT"),
        }
    }
}
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);
pub type RealId = String;
pub type FmStepId = String;
