use std::sync::Arc;

use log::{debug, info, warn};

use crate::{
    evaluator::{EEFullSnapshotEvent, ZMQIdentity},
    EE,
};

impl EE {
    pub fn handle_ert(self: &Arc<Self>, ert_zmq_id: &ZMQIdentity, ert_id: &String, frame: &String) {
        if frame == "CONNECT" {
            let mut n = self._ert_identity.write().unwrap();
            *n = Some(ert_zmq_id.clone());
            info!("CONNECTED {}", ert_id);
        } else if frame == "DISCONNECT" {
            *self._ert_identity.write().unwrap() = None;
            info!("DISCONNECTED {}", ert_id);
        } else {
            self._handle_event_from_ert(frame);
        }
    }

    pub fn handle_dispatch(
        self: &Arc<Self>,
        dispatcher_zmq_id: &ZMQIdentity,
        dealer_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .insert(dispatcher_zmq_id.clone());
            debug!("CONNECTED {}", dealer_id);
        } else if frame == "DISCONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .remove(dispatcher_zmq_id);
            debug!("DISCONNECTED {}", dealer_id);
        } else {
            self._handle_event_from_dispatcher(frame)
        }
    }

    pub fn handle_client(
        self: &Arc<Self>,
        sender_identity: &ZMQIdentity,
        client_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            {
                if self
                    ._client_connected
                    .read()
                    .unwrap()
                    .contains(sender_identity)
                {
                    warn!("Client '{}' wants to reconnect", client_id);
                }
            }

            {
                self._client_connected
                    .write()
                    .unwrap()
                    .insert(sender_identity.clone());
                debug!("CONNECTED {}", client_id);
            }
            let full_snapshot_event = EEFullSnapshotEvent::new(
                self._main_snapshot.read().unwrap().clone(),
                self._ensemble_id
                    .read()
                    .unwrap()
                    .clone()
                    .unwrap_or_default(),
            );
            {
                // might not need the extra block here. It was just to make sure the lock was released
                self._send_bytes_to_identity(
                    &sender_identity,
                    &serde_json::to_string(&full_snapshot_event)
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                );
                info!("SENT FULL SNAPSHOT!")
            }
        } else if frame == "DISCONNECT" {
            {
                self._client_connected
                    .write()
                    .unwrap()
                    .remove(sender_identity);
                debug!("DISCONNECTED {}", client_id);
            }
        } else {
            self._handle_event_from_client(frame)
        }
    }
}
