use std::{sync::Arc, thread, time::Duration};

use serde::Serialize;

use crate::{evaluator::QueueEvents, EE};
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);
#[derive(Debug, Clone, Serialize)]
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

impl EE {
    pub fn _do_heartbeat_clients(self: Arc<Self>) {
        while self.is_running() {
            if self._client_connected.read().unwrap().is_empty() {
                thread::sleep(Duration::from_millis(100));
            } else {
                self._events_to_send
                    .push(QueueEvents::HeartBeat(HeartBeat::new()));
                thread::sleep(HEARTBEAT_TIMEOUT);
            }
        }
    }
}
