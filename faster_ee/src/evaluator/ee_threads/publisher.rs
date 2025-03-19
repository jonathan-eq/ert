use log::{debug, error, info};

use super::super::QueueEvents;
use super::do_heartbeat_clients::HEARTBEAT;
use crate::events::client_event::ClientEvent;
use crate::events::dispatcher_event::checksum_event::ForwardModelStepChecksum;
use crate::events::dispatcher_event::DispatcherEvent;
use crate::events::ert_event::ErtEvent;
use crate::events::snapshot_event::EESnapshotEvent;
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
                    self._handle_checksum_event(&ert_identity, inner);
                } else {
                    error!("Found no Ert identity to forward checksum to");
                }
            }
            QueueEvents::HeartBeat => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    self._handle_heartbeat_event(&ert_identity);
                }
            }
            QueueEvents::EnsembleSnapshot(inner) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    let json_str =
                        &serde_json::to_string(&ErtEvent::EESnapshotUpdate(inner.clone())).unwrap();
                    debug!("Sending EESnapshotUpdate to Ert identity {}", json_str);
                    self._send_bytes_to_identity(&ert_identity, &json_str.as_bytes().to_vec());
                    debug!("Finished sending EESnapshot")
                }
            }
            QueueEvents::UserCancelledEE(inner) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    let json_str = &serde_json::to_string(&inner).unwrap();
                    debug!("Sending UserCancelledEE to Ert identity {}", json_str);
                    self._send_bytes_to_identity(&ert_identity, &json_str.as_bytes().to_vec());
                }
            }
            QueueEvents::UserDone(inner_event) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    let json_str =
                        &serde_json::to_string(&ClientEvent::EEUserDone(inner_event.clone()))
                            .unwrap();
                    debug!("Sending UserDone to Ert identity {}", json_str);
                    self._send_bytes_to_identity(&ert_identity, &json_str.as_bytes().to_vec());
                }
            }
            QueueEvents::FullEnsembleSnapshot(inner) => {
                if let Some(ert_identity) = self._ert_identity.read().unwrap().clone() {
                    let json_str =
                        &serde_json::to_string(&ErtEvent::EEFullSnapshot(inner.clone())).unwrap();
                    debug!("Sending Full Snapshot to Ert identity {}", json_str);
                    self._send_bytes_to_identity(&ert_identity, &json_str.as_bytes().to_vec());
                    debug!("Finished sending EESnapshot")
                }
            }
        }
    }
    pub fn _publisher(self: Arc<Self>) {
        while self.is_running() | !self._events_to_send.is_empty() {
            if let Some(event) = self._events_to_send.pop() {
                self._inform_ert(&event);

                let identities = { self._client_connected.read().unwrap().clone() };

                for identity in &identities {
                    match &event {
                        QueueEvents::HeartBeat => self._handle_heartbeat_event(identity),
                        QueueEvents::EnsembleSnapshot(event) => {
                            self._handle_snapshot_event(identity, event)
                        }
                        QueueEvents::Checksum(event) => {
                            self._handle_checksum_event(identity, event)
                        }
                        QueueEvents::UserCancelledEE(_) => {}
                        QueueEvents::UserDone(_) => {}
                        QueueEvents::FullEnsembleSnapshot(event) => {
                            self._handle_full_snapshot_event(identity, event)
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
        snapshot_event: &EESnapshotEvent,
    ) {
        debug!("SENDING EESNAPSHOT UPDATE TO CLIENT");
        match &serde_json::to_string(&ErtEvent::EESnapshotUpdate(snapshot_event.clone())) {
            Ok(snapshot_event_str) => {
                self._send_bytes_to_identity(identity, &snapshot_event_str.as_bytes().to_vec());
                debug!("FINISHED SENDING EESNAPSHOT UPDATE TO CLIENT");
            }
            Err(err) => {
                error!("Failed deserializing EESnapshotEvent {}", err.to_string());
            }
        }
    }
    fn _handle_full_snapshot_event(
        self: &Arc<Self>,
        identity: &Vec<u8>,
        snapshot_event: &EESnapshotEvent,
    ) {
        debug!("SENDING FULL EESNAPSHOT TO CLIENT");
        match &serde_json::to_string(&ErtEvent::EEFullSnapshot(snapshot_event.clone())) {
            Ok(snapshot_event_str) => {
                self._send_bytes_to_identity(identity, &snapshot_event_str.as_bytes().to_vec());
                debug!("FINISHED SENDING FULL EESNAPSHOT TO CLIENT");
            }
            Err(err) => {
                error!("Failed deserializing EESnapshotEvent {}", err.to_string());
            }
        }
    }
    fn _handle_heartbeat_event(self: &Arc<Self>, identity: &Vec<u8>) {
        self._send_bytes_to_identity(identity, &HEARTBEAT.to_vec());
    }
    fn _handle_checksum_event(
        self: &Arc<Self>,
        identity: &Vec<u8>,
        checksum_event: &ForwardModelStepChecksum,
    ) {
        match serde_json::to_string(&DispatcherEvent::ForwardModelStepChecksum(
            checksum_event.clone(),
        )) {
            Ok(checksum_event_str) => {
                self._send_bytes_to_identity(identity, &checksum_event_str.as_bytes().to_vec());
            }
            Err(err) => {
                error!("Failed serializing checksum {}", err.to_string());
            }
        }
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
