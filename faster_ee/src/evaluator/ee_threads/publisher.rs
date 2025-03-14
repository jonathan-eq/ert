use log::{debug, error, info};

use super::super::QueueEvents;
use super::do_heartbeat_clients::HeartBeat;
use crate::evaluator::EESnapshotUpdateEvent;
use crate::events::dispatcher_event::checksum_event::ForwardModelStepChecksum;
use crate::EE;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

impl EE {
    pub fn _inform_ert(self: &Arc<Self>, event: &QueueEvents) {
        match event {
            QueueEvents::Checksum(inner) => {
                // Send checksum to ert
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    info!("Sending checksum to Ert identity");
                    self._send_bytes_to_identity(
                        &ert_identity,
                        &serde_json::to_string(&inner).unwrap().as_bytes().to_vec(),
                    );
                } else {
                    error!("Found no Ert identity to forward checksum to");
                }
            }
            QueueEvents::HeartBeat(inner) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    self._handle_heartbeat_event(&ert_identity, inner);
                }
            }
            QueueEvents::EnsembleSnapshot(inner) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    let json_str = &serde_json::to_string(&inner).unwrap();
                    debug!("Sending EESnapshotUpdate to Ert identity {}", json_str);
                    self._send_bytes_to_identity(&ert_identity, &json_str.as_bytes().to_vec());
                }
            }
        }
    }
    pub fn _publisher(self: Arc<Self>) {
        while self.is_running() {
            if let Some(event) = self._events_to_send.pop() {
                self._inform_ert(&event);

                let identities = { self._client_connected.read().unwrap().clone() };

                for identity in &identities {
                    match &event {
                        QueueEvents::HeartBeat(event) => {
                            self._handle_heartbeat_event(identity, event)
                        }
                        QueueEvents::EnsembleSnapshot(event) => {
                            self._handle_snapshot_event(identity, event)
                        }
                        QueueEvents::Checksum(event) => {
                            self._handle_checksum_event(identity, event)
                        }
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
    fn _handle_snapshot_event(
        self: &Arc<Self>,
        identity: &Vec<u8>,
        snapshot_event: &EESnapshotUpdateEvent,
    ) {
        self._send_bytes_to_identity(
            identity,
            &serde_json::to_string(&snapshot_event)
                .unwrap()
                .as_bytes()
                .to_vec(),
        );
    }
    fn _handle_heartbeat_event(self: &Arc<Self>, identity: &Vec<u8>, heartbeat_event: &HeartBeat) {
        self._send_bytes_to_identity(identity, &heartbeat_event.msg.as_bytes().to_vec());
    }
    fn _handle_checksum_event(
        self: &Arc<Self>,
        identity: &Vec<u8>,
        checksum_event: &ForwardModelStepChecksum,
    ) {
        self._send_bytes_to_identity(
            identity,
            &serde_json::to_string(checksum_event)
                .unwrap()
                .as_bytes()
                .to_vec(),
        );
    }
    pub fn _send_bytes_to_identity(self: &Arc<Self>, identity: &Vec<u8>, bytes: &Vec<u8>) {
        let socket = self._router_socket.lock().unwrap();
        socket
            .as_ref()
            .unwrap()
            .send_multipart([identity, &vec![], bytes].iter(), 0)
            .unwrap();
    }
}
