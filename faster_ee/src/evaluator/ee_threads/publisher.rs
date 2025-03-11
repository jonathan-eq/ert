use super::super::QueueEvents;
use super::do_heartbeat_clients::HeartBeat;
use crate::evaluator::EESnapshotUpdateEvent;
use crate::EE;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

impl EE {
    pub fn _publisher(self: Arc<Self>) {
        while self.is_running() {
            if let Some(event) = self._events_to_send.pop() {
                println!("PUBLISHER FOUND EVENT TO SEND!");
                let identities = { self._client_connected.read().unwrap().clone() };
                if identities.len() == 0 {
                    eprintln!("FOUND NO IDENTITIES :(")
                }
                for identity in &identities {
                    println!("FOUND IDENTITY TO SEND TO");
                    match &event {
                        QueueEvents::HeartBeat(event) => {
                            self._handle_heartbeat_event(identity, event)
                        }
                        QueueEvents::EnsembleSnapshot(event) => {
                            self._handle_snapshot_event(identity, event)
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
        //println!("SENDING SNAPSHOT TO CLIENT! {:#?}", snapshot_event);
        let socket = self._router_socket.lock().unwrap();
        socket
            .as_ref()
            .unwrap()
            .send_multipart(
                vec![
                    identity,
                    &vec![],
                    &serde_json::to_string(&snapshot_event)
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                ]
                .iter(),
                0,
            )
            .unwrap();
        //println!("SENT SNAPSHOT TO CLIENT!");
    }
    fn _handle_heartbeat_event(self: &Arc<Self>, identity: &Vec<u8>, heartbeat_event: &HeartBeat) {
        println!("GOT HEARTBEAT TO SEND!");
        let socket = self._router_socket.lock().unwrap();
        socket
            .as_ref()
            .unwrap()
            .send_multipart(
                [identity, &vec![], heartbeat_event.msg.as_bytes()].iter(),
                0,
            )
            .unwrap();
    }
}
