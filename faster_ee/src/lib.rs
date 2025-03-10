use chrono::{DateTime, Utc};
use crossbeam::queue::SegQueue;
use events::snapshots::fm_step_snapshot::FMEvent;
use events::types::{HeartBeat, DISCONNECT_MSG, HEARTBEAT_TIMEOUT};
use pyo3::prelude::*;
pub mod events;
pub mod utils;
use events::snapshots::ensemble_snapshot::{ClientEvent, DispatcherEvent, EnsembleSnapshot, Event};
use events::{snapshot::*, types::Id};
/// A Python module implemented in Rust.
// #[pymodule]
// fn faster_ee(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<EE>()?;
//     Ok(())
// }
#[derive(Eq, Hash, PartialEq)]
enum DestinationHandler {
    FMHandler,
    EnsembleStarted,
    EnsembleSucceeded,
    EnsembleCancelled,
    EnsembleFailed,
}
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::{collections::HashSet, thread, time::Duration};

#[derive(Clone)]
pub struct EE {
    _thread: Arc<Option<std::thread::JoinHandle<()>>>,
    address: Arc<String>,
    is_running: Arc<AtomicBool>,
    _client_connected: Arc<RwLock<HashSet<String>>>, // Cannot use Bytes due to pyclass restrictions
    _dispatchers_connected: Arc<RwLock<HashSet<String>>>,
    _events_to_send: Arc<SegQueue<QueueEvents>>,
    _batch_processing_queue: Arc<SegQueue<HashMap<DestinationHandler, Vec<Event>>>>,
    _events: Arc<SegQueue<DispatcherEvent>>,
    _router_socket: Arc<Mutex<Option<zmq::Socket>>>,
    pub _ensemble_id: Arc<String>,
    _main_snapshot: Arc<RwLock<EnsembleSnapshot>>,
    _batching_interval: Arc<Duration>,
    _max_batch_size: Arc<i64>,
    _ensemble_status: Arc<String>,
    server_curve: Arc<Option<(Vec<u8>, Vec<u8>)>>,
    is_socket_ready: Arc<AtomicBool>,
}
enum QueueEvents {
    HeartBeat(HeartBeat),
    EnsembleSnapshot(EESnapshotUpdate),
}

impl EE {
    //#[new]
    //#[pyo3(signature=(address, server_curve, ensemble_id))]
    pub fn new(
        address: String,
        server_curve: Option<(Vec<u8>, Vec<u8>)>,
        ensemble_id: String,
    ) -> Self {
        let instance = Self {
            _thread: Arc::new(None),
            address: Arc::new(address),
            is_running: Arc::new(AtomicBool::new(true)),
            _client_connected: Arc::new(RwLock::new(HashSet::new())),
            _dispatchers_connected: Arc::new(RwLock::new(HashSet::new())),
            _events_to_send: Arc::new(SegQueue::new()),
            _router_socket: Arc::new(Mutex::new(None)),
            _ensemble_id: Arc::new(ensemble_id),
            _main_snapshot: Arc::new(RwLock::new(EnsembleSnapshot::default())),
            _max_batch_size: Arc::new(500),
            _batch_processing_queue: Arc::new(SegQueue::new()),
            _events: Arc::new(SegQueue::new()),
            _batching_interval: Arc::new(Duration::from_secs(1)),
            _ensemble_status: Arc::new(String::from("Running")),
            server_curve: Arc::new(server_curve),
            is_socket_ready: Arc::new(AtomicBool::new(false)),
        };
        instance
    }

    pub fn run(self: Arc<Self>) {
        //thread::spawn(move || self_clone._server())
        //let self_clone = Arc::new(self);
        //let self_clone = Arc::new(self);

        match self._run() {
            Ok(_) => {
                println!("SUCCEEDED AND FINISHED EE");
            }
            Err(inner_msg) => {
                eprintln!("{}", inner_msg);
            }
        }

        //a.join();
    }

    //fn wait_for_finish(&mut self) -> Option<()> {
    //    self._thread
    //        .take()
    //        .map(|inner_thread| inner_thread.join().ok())
    //        .flatten()
    //}
}

impl EE {
    fn is_running(self: &Arc<Self>) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
    fn _do_heartbeat_clients(self: Arc<Self>) {
        while self.is_running() {
            if self._client_connected.read().unwrap().is_empty() {
                thread::sleep(Duration::from_millis(100));
            } else {
                self._events_to_send
                    .push(QueueEvents::HeartBeat(HeartBeat::new()));
                thread::sleep(HEARTBEAT_TIMEOUT);
            }
        }
    }

    fn _publisher(self: Arc<Self>) {
        while self.is_running() {
            match self._events_to_send.pop() {
                Some(event) => {
                    let identities = self._client_connected.read().unwrap().clone();
                    let socket = self._router_socket.lock().unwrap();
                    for identity in identities {
                        match event {
                            QueueEvents::HeartBeat(ref heartbeat_event) => {
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

    fn _append_message(self: &Arc<Self>, snapshot_update_event: EnsembleSnapshot) {
        let event = EESnapshotUpdate {
            snapshot: snapshot_update_event,
            ensemble: self._ensemble_id.to_string().clone(),
        };
        self._events_to_send
            .push(QueueEvents::EnsembleSnapshot(event));
    }
    fn _cancelled_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() != "FAILED" {
            let update_snapshot_event =
                self._main_snapshot.read().unwrap().update_snapshot(&events);
            self._append_message(update_snapshot_event);
            self.stop();
        }
    }
    fn stop(self: &Arc<Self>) {
        self.is_running.store(false, Ordering::SeqCst);
    }
    fn _started_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() != "Failed" {
            let update_snapshot_event = self._main_snapshot.read().unwrap().update_snapshot(events);
            self._append_message(update_snapshot_event);
        }
    }
    fn _failed_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if (self._ensemble_status.to_string() == "Failed")
            | (self._ensemble_status.to_string() == "Cancelled")
        {
            return;
        }
        let snapshot_update_event = self._main_snapshot.read().unwrap().update_snapshot(events);
        self._append_message(snapshot_update_event);
        self._signal_cancel();
    }
    fn _signal_cancel(self: &Arc<Self>) {
        eprintln!("SIGNAL_CANCEL NOT IMPLEMENTED YET!");
    }
    fn _stopped_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() == "Failed" {
            return;
        }
        let snapshot_update_event = self._main_snapshot.read().unwrap().update_snapshot(events);
        self._append_message(snapshot_update_event);
    }

    fn process_event_buffer(self: Arc<Self>) {
        while self.is_running() {
            match self._batch_processing_queue.pop() {
                Some(inner_event) => {
                    for (handler, events) in inner_event {
                        match handler {
                            DestinationHandler::FMHandler => {
                                self._fm_handler(&events);
                            }
                            DestinationHandler::EnsembleCancelled => {
                                self._cancelled_handler(&events);
                            }
                            DestinationHandler::EnsembleStarted => {
                                self._started_handler(&events);
                            }
                            DestinationHandler::EnsembleFailed => {
                                self._failed_handler(&events);
                            }
                            DestinationHandler::EnsembleSucceeded => {
                                self._stopped_handler(&events);
                            }
                        };
                    }
                }
                None => {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }

    fn _batch_events_into_buffer(self: Arc<Self>) {
        while self.is_running() {
            let mut batch: HashMap<DestinationHandler, Vec<Event>> = HashMap::new();
            let start_time = Utc::now();
            let mut events_in_map_count: i64 = 0;
            while (events_in_map_count < *self._max_batch_size)
                && (Utc::now() < start_time + *self._batching_interval)
            {
                events_in_map_count = events_in_map_count + 1;
                match self._events.pop() {
                    Some(inner_event) => match inner_event {
                        DispatcherEvent::ForwardModelStepFailure(nested) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(Event::FMEvent(FMEvent::ForwardModelStepFailure(nested)));
                        }
                        _ => {
                            println!("Not handling this type of event yet")
                        }
                    },
                    None => {
                        thread::sleep(Duration::from_millis(100));
                        events_in_map_count = events_in_map_count - 1
                    }
                }
            }
            self._batch_processing_queue.push(batch);
            if self._events.len() > 500 {
                println!(
                    "There is a lot of events left in queue ({})",
                    self._events.len()
                )
            }
        }
    }

    fn _handle_event_from_dispatcher(self: &Arc<Self>, json_string: String) {
        match serde_json::from_str::<DispatcherEvent>(json_string.as_str()) {
            Ok(event) => {
                self._events.push(event);
            }
            Err(err) => {
                eprintln!("{:?}", err)
            }
        }
    }
    fn _handle_event_from_client(self: &Arc<Self>, json_string: String) {
        match serde_json::from_str::<ClientEvent>(json_string.as_str()) {
            Ok(event) => match event {
                ClientEvent::EEUserCancel(_) => {
                    println!("Client asked to cancel.");
                    self._signal_cancel();
                }
                ClientEvent::EEUserDone(_) => {
                    println!("Client signalled done");
                    self.stop();
                }
            },
            Err(err) => {
                eprintln!("handle_client failed {:?}", err)
            }
        }
    }

    fn handle_dispatch(self: &Arc<Self>, dealer_id: &String, frame: String) {
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
            self._handle_event_from_dispatcher(frame)
        }
    }

    fn handle_client(
        self: &Arc<Self>,
        sender_identity: Vec<u8>,
        client_id: &String,
        frame: String,
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
            let full_snapshot_event = EESnapshotUpdate {
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

    fn listen_for_messages(self: Arc<Self>) {
        while !self.is_socket_ready.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        while self.is_running() {
            thread::sleep(Duration::from_millis(500));
            println!("RUNNING LISTEN_FOR_MESSAGES");

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
                println!("GOT MESSAGE!");

                // Clone message parts so we don't hold references
                let sender = inbound_msg.get(0).unwrap().clone();
                let sender_id = inbound_msg.get(1).unwrap().clone();
                let payload = inbound_msg.get(3).unwrap().clone();

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
                println!("ESTABLISHED CONNECTION WITH {}", decoded_sender);

                // Handle message
                if decoded_sender.starts_with("client") {
                    self.handle_client(sender, &decoded_sender, decoded_payload);
                } else if decoded_sender.starts_with("dispatch") {
                    self.handle_dispatch(&decoded_sender, decoded_payload);
                } else {
                    eprintln!("Received msg from unknown sender '{}'", &decoded_sender);
                }
            }
        }
        println!("EXITING LISTEN_FOR_MESSAGES");
    }

    fn _fm_handler(self: &Arc<Self>, events: &Vec<Event>) {
        let update_snapshot_event = self._main_snapshot.read().unwrap().update_snapshot(&events);
        self._append_message(update_snapshot_event);
    }
    fn _server(self: Arc<Self>) {
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
    fn _run(self: Arc<Self>) -> Result<(), String> {
        let main_clone = self;
        let server_thread = {
            let self_clone = Arc::clone(&main_clone);
            thread::spawn(move || self_clone._server())
        };
        //let heartbeat_thread = {
        //    let self_clone = Arc::clone(&main_clone);
        //    thread::spawn(move || self_clone._do_heartbeat_clients())
        //};
        let _ = server_thread.join();
        let listen_for_messages_thread = {
            let self_clone = Arc::clone(&main_clone);
            thread::Builder::new()
                .name("listen_for_messages_thread".to_string())
                .spawn(move || self_clone.listen_for_messages())
                .unwrap()
        };
        //let process_event_buffer_thread = {
        //    let self_clone = Arc::clone(&main_clone);
        //    thread::spawn(move || self_clone.process_event_buffer())
        //};

        //let _ = heartbeat_thread.join();
        let _ = listen_for_messages_thread.join();
        //let _ = process_event_buffer_thread.join();
        Ok(())
    }
}
