use std::{sync::Arc, thread, time::Duration};

use crate::{evaluator::QueueEvents, EE};
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);
pub const HEARTBEAT: &[u8; 4] = b"BEAT";

impl EE {
    pub fn _do_heartbeat_clients(self: Arc<Self>) {
        while self.is_running() {
            if !self._client_connected.read().unwrap().is_empty() {
                self._events_to_send.push(QueueEvents::HeartBeat);
            }
            thread::sleep(HEARTBEAT_TIMEOUT);
        }
    }
}
