pub mod connection_handlers;
pub mod event_handlers;

use std::{
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

use log::debug;

use crate::EE;

impl EE {
    pub fn listen_for_messages(self: Arc<Self>) {
        while !self.is_socket_ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        while self.is_running() {
            thread::sleep(Duration::from_millis(500));

            let inbound_msg = {
                let socket_lock = self._router_socket.lock().unwrap();
                let socket = socket_lock.as_ref().unwrap();
                match socket.recv_multipart(zmq::DONTWAIT) {
                    Ok(msg) => Some(msg),
                    Err(_) => None,
                }
            };

            // If we received a message, process it
            if let Some(inbound_msg) = inbound_msg {
                self.handle_message(inbound_msg);
            }
        }
    }
    pub fn handle_message(self: &Arc<Self>, message: Vec<Vec<u8>>) {
        // Clone message parts so we don't hold references
        let sender_zmq_id = message.get(0).unwrap().clone();
        let sender_id = message.get(1).unwrap().clone();
        let payload = message.get(3).unwrap().clone(); // There are some problems communicating between python zmq and rust...

        self._send_bytes_to_identity(&sender_zmq_id, &b"ACK".to_vec());

        // Decode message
        let decoded_sender = String::from_utf8_lossy(&sender_id).to_string();
        let decoded_payload = String::from_utf8_lossy(&payload).to_string();

        // Handle message
        if decoded_sender.starts_with("client") {
            self.handle_client(&sender_zmq_id, &decoded_sender, &decoded_payload);
        } else if decoded_sender.starts_with("dispatch") {
            self.handle_dispatch(&sender_zmq_id, &decoded_sender, &decoded_payload);
        } else if decoded_sender.starts_with("ert") {
            self.handle_ert(&sender_zmq_id, &decoded_sender, &decoded_payload);
        } else {
            eprintln!("Received msg from unknown sender '{}'", &decoded_sender);
        }
    }
}
