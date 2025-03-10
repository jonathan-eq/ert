use super::super::QueueEvents;
use super::do_heartbeat_clients::HeartBeat;
use crate::EE;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

impl EE {
    fn _publisher(self: Arc<Self>) {
        while self.is_running() {
            match self._events_to_send.pop() {
                Some(event) => {
                    let identities = self._client_connected.read().unwrap().clone();
                    let socket = self._router_socket.lock().unwrap();
                    for identity in identities {
                        match event {
                            QueueEvents::HeartBeat(ref heartbeat_event) => {
                                println!("GOT HEARTBEAT TO SEND!");
                                socket
                                    .as_ref()
                                    .unwrap()
                                    .send_multipart(
                                        vec![
                                            identity.as_bytes(),
                                            &[].to_vec(),
                                            heartbeat_event.msg.as_bytes(),
                                        ]
                                        .iter(),
                                        0,
                                    )
                                    .unwrap();
                            }
                            QueueEvents::EnsembleSnapshot(ref inner_event) => {
                                socket
                                    .as_ref()
                                    .unwrap()
                                    .send_multipart(
                                        vec![
                                            identity.as_bytes(),
                                            &[].to_vec(),
                                            serde_json::to_string(&inner_event.snapshot)
                                                .unwrap()
                                                .as_bytes(),
                                        ]
                                        .iter(),
                                        0,
                                    )
                                    .unwrap();
                            }
                        }
                    }
                }
                None => {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }
}
