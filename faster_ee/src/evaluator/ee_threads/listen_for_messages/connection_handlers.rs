use std::sync::Arc;

use log::{debug, info, warn};

use crate::{
    evaluator::ZMQIdentity,
    events::{ert_event::ErtEvent, snapshot_event::EESnapshotEvent},
    EE,
};

impl EE {
    pub fn handle_ert(self: &Arc<Self>, ert_zmq_id: &ZMQIdentity, ert_id: &String, frame: &String) {
        if frame == "CONNECT" {
            {
                let mut n = self._ert_identity.write().unwrap();
                *n = Some(ert_zmq_id.clone());
                info!("CONNECTED {}", ert_id);
            }
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
        dispatcher_name: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            self._handle_dispatcher_connect(dispatcher_zmq_id, dispatcher_name);
        } else if frame == "DISCONNECT" {
            self._handle_dispatcher_disconnect(dispatcher_zmq_id, dispatcher_name);
        } else {
            self._handle_event_from_dispatcher(frame)
        }
    }
    pub fn _handle_dispatcher_connect(
        self: &Arc<Self>,
        dispatcher_zmq_id: &ZMQIdentity,
        dispatcher_name: &String,
    ) {
        self._dispatchers_connected
            .write()
            .unwrap()
            .insert(dispatcher_zmq_id.clone());
        debug!("CONNECTED {}", dispatcher_name);
    }

    pub fn _handle_dispatcher_disconnect(
        self: &Arc<Self>,
        dispatcher_zmq_id: &ZMQIdentity,
        dispatcher_name: &String,
    ) {
        self._dispatchers_connected
            .write()
            .unwrap()
            .remove(dispatcher_zmq_id);
        debug!("DISCONNECTED {}", dispatcher_name);
    }
    pub fn handle_client(
        self: &Arc<Self>,
        client_zmq_id: &ZMQIdentity,
        client_name: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            self._handle_client_connect(client_name, client_zmq_id);
        } else if frame == "DISCONNECT" {
            self._handle_client_disconnect(client_name, client_zmq_id);
        } else {
            self._handle_event_from_client(frame)
        }
    }
    pub fn _handle_client_disconnect(
        self: &Arc<Self>,
        client_name: &String,
        client_zmq_id: &ZMQIdentity,
    ) {
        self._client_connected
            .write()
            .unwrap()
            .remove(client_zmq_id);
        debug!("DISCONNECTED CLIENT {}", client_name);
    }
    pub fn _handle_client_connect(
        self: &Arc<Self>,
        client_name: &String,
        client_zmq_id: &ZMQIdentity,
    ) {
        if self
            ._client_connected
            .read()
            .unwrap()
            .contains(client_zmq_id)
        {
            warn!("Client '{}' wants to reconnect", client_name);
        }

        {
            self._client_connected
                .write()
                .unwrap()
                .insert(client_zmq_id.clone());
            debug!("CONNECTED {}", client_name);
        }
        let full_snapshot_event = EESnapshotEvent::new(
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
                &client_zmq_id,
                &serde_json::to_string(&ErtEvent::EEFullSnapshot(full_snapshot_event))
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            );
            info!("SENT FULL SNAPSHOT!")
        }
    }
}
