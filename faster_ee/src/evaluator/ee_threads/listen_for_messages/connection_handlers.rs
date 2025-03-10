use std::sync::Arc;

use crate::{events::snapshot_event::EESnapshotUpdateEvent, EE};

impl EE {
    pub fn handle_dispatch(
        self: &Arc<Self>,
        dispatcher_zmq_id: Vec<u8>,
        dealer_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .insert(dealer_id.clone());
        } else if frame == "DISCONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .remove(dealer_id);
        } else {
            println!("INVOKING EVENT FROM DISPATCHER HANDLER");
            self._handle_event_from_dispatcher(frame)
        }
    }

    pub fn handle_client(
        self: &Arc<Self>,
        sender_identity: Vec<u8>,
        client_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            println!("HANDLIGN CONNECT");
            if self._client_connected.read().unwrap().contains(client_id) {
                eprintln!("Client '{}' wants to reconnect", client_id);
            }

            self._client_connected
                .write()
                .unwrap()
                .insert(client_id.clone());
            let full_snapshot_event = EESnapshotUpdateEvent {
                snapshot: self._main_snapshot.read().unwrap().clone(),
                ensemble: self._ensemble_id.to_string().clone(),
            };
            let socket = self._router_socket.lock().unwrap();
            socket
                .as_ref()
                .unwrap()
                .send_multipart(
                    [
                        sender_identity,
                        client_id.as_bytes().to_vec(),
                        [].to_vec(),
                        serde_json::to_string(&full_snapshot_event)
                            .unwrap()
                            .as_bytes()
                            .to_vec(),
                    ]
                    .iter(),
                    0,
                )
                .unwrap();
            println!("SENT FULL SNAPSHOT!")
        } else if frame == "DISCONNECT" {
            self._client_connected.write().unwrap().remove(client_id);
        } else {
            println!("DONT KNOW WHAT I GOT {}", frame);
            self._handle_event_from_client(frame)
        }
    }
}
