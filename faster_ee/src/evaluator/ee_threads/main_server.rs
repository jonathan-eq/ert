use std::sync::{atomic::Ordering, Arc};

use crate::EE;

impl EE {
    pub fn _server(self: Arc<Self>) {
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::ROUTER).unwrap();
        socket.bind(&self.address).unwrap();
        socket.set_linger(1).unwrap();
        socket.set_identity("faster_ee".as_bytes()).unwrap();
        if let Some((server_public_key, server_secret_key)) = &*self.server_curve {
            socket.set_curve_server(true).unwrap();
            socket.set_curve_serverkey(&server_secret_key).unwrap();
            socket.set_curve_publickey(&server_public_key).unwrap();
        }
        {
            let mut lock = self._router_socket.lock().unwrap();
            *lock = Some(socket);
        }
        self.is_socket_ready.store(true, Ordering::SeqCst);
        println!("FINISHED starting server");
    }
}
