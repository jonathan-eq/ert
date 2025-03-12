pub mod connection_handlers;
pub mod event_handlers;

use std::{
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

use crate::EE;

impl EE {
    pub fn listen_for_messages(self: Arc<Self>) {
        while !self.is_socket_ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        while self.is_running() {
            thread::sleep(Duration::from_millis(500));

            // 🔓 Get the message while holding the lock
            let inbound_msg = {
                let socket_lock = self._router_socket.lock().unwrap();
                let socket = socket_lock.as_ref().unwrap();
                match socket.recv_multipart(zmq::DONTWAIT) {
                    Ok(msg) => Some(msg),
                    Err(inner_err) => {
                        //eprintln!("{:?}", inner_err);
                        None
                    }
                }
            }; // 🔓 Lock released here ✅

            // If we received a message, process it
            if let Some(inbound_msg) = inbound_msg {
                // Clone message parts so we don't hold references
                let sender = inbound_msg.get(0).unwrap().clone();
                let sender_id = inbound_msg.get(1).unwrap().clone();
                let payload = inbound_msg.get(3).unwrap().clone(); // There are some problems communicating between python zmq and rust...

                // 🔓 Lock again to send the response
                {
                    let socket_lock = self._router_socket.lock().unwrap();
                    let socket = socket_lock.as_ref().unwrap();
                    socket
                        .send_multipart(vec![&sender, &vec![], &b"ACK".to_vec()].iter(), 0)
                        .unwrap();
                } // 🔓 Lock released again ✅

                // Decode message
                let decoded_sender = String::from_utf8_lossy(&sender_id).to_string();
                let decoded_payload = String::from_utf8_lossy(&payload).to_string();
                //println!("ESTABLISHED CONNECTION WITH {}", decoded_sender);

                // Handle message
                if decoded_sender.starts_with("client") {
                    self.handle_client(&sender, &decoded_sender, &decoded_payload);
                } else if decoded_sender.starts_with("dispatch") {
                    self.handle_dispatch(&sender, &decoded_sender, &decoded_payload);
                } else if decoded_sender.starts_with("ert") {
                    self.handle_ert(&sender, &decoded_sender, &decoded_payload);
                } else {
                    eprintln!("Received msg from unknown sender '{}'", &decoded_sender);
                }
            }
        }
        println!("EXITING LISTEN_FOR_MESSAGES");
    }
}
