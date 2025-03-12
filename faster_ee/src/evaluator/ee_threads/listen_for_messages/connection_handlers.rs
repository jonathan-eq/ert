use std::sync::Arc;

use crate::{evaluator::EEFullSnapshotEvent, EE};

impl EE {
    pub fn handle_dispatch(
        self: &Arc<Self>,
        dispatcher_zmq_id: &Vec<u8>,
        dealer_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .insert(dispatcher_zmq_id.clone());
            println!("CONNECTED {}", dealer_id);
        } else if frame == "DISCONNECT" {
            self._dispatchers_connected
                .write()
                .unwrap()
                .remove(dispatcher_zmq_id);
            println!("DISCONNECTED {}", dealer_id);
        } else {
            println!("INVOKING EVENT FROM DISPATCHER HANDLER");
            self._handle_event_from_dispatcher(frame)
        }
    }

    pub fn handle_client(
        self: &Arc<Self>,
        sender_identity: &Vec<u8>,
        client_id: &String,
        frame: &String,
    ) {
        if frame == "CONNECT" {
            println!("HANDLIGN CONNECT");
            {
                if self
                    ._client_connected
                    .read()
                    .unwrap()
                    .contains(sender_identity)
                {
                    eprintln!("Client '{}' wants to reconnect", client_id);
                }
            }

            {
                self._client_connected
                    .write()
                    .unwrap()
                    .insert(sender_identity.clone());
                println!("CONNECTED {}", client_id);
            }
            let full_snapshot_event = EEFullSnapshotEvent {
                snapshot: self._main_snapshot.read().unwrap().clone(),
                ensemble: self._ensemble_id.to_string().clone(),
            };
            //println!("{:?}", full_snapshot_event);
            {
                let socket = self._router_socket.lock().unwrap();
                socket
                    .as_ref()
                    .unwrap()
                    .send_multipart(
                        vec![
                            &sender_identity,
                            &vec![],
                            &serde_json::to_string(&full_snapshot_event)
                                .unwrap()
                                .as_bytes()
                                .to_vec(),
                        ]
                        .iter(),
                        0,
                    )
                    .unwrap();
                println!("SENT FULL SNAPSHOT!")
            }
        } else if frame == "DISCONNECT" {
            {
                self._client_connected
                    .write()
                    .unwrap()
                    .remove(sender_identity);
                println!("DISCONNECTED {}", client_id);
            }
        } else {
            println!("DONT KNOW WHAT I GOT {}", frame);
            self._handle_event_from_client(frame)
        }
    }
}
