use crate::events::dispatcher_event::DispatcherEvent;

use crate::events::Event;

use crossbeam::queue::SegQueue;
use ee_threads::do_heartbeat_clients::HeartBeat;
use ee_threads::DestinationHandler;

use crate::events::snapshot_event::*;
use crate::snapshots::EnsembleSnapshot;
mod ee_threads;
pub mod handlers;
/// A Python module implemented in Rust.
// #[pymodule]
// fn faster_ee(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<EE>()?;
//     Ok(())
// }
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
    EnsembleSnapshot(EESnapshotUpdateEvent),
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
        match self._run() {
            Ok(_) => {
                println!("SUCCEEDED AND FINISHED EE");
            }
            Err(inner_msg) => {
                eprintln!("{}", inner_msg);
            }
        }
    }
}

impl EE {
    fn is_running(self: &Arc<Self>) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn stop(self: &Arc<Self>) {
        self.is_running.store(false, Ordering::SeqCst);
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
        let _batch_events_into_buffer_thread = {
            let self_clone = Arc::clone(&main_clone);
            thread::Builder::new()
                .name("batch_events_into_buffer_thread".to_string())
                .spawn(move || self_clone._batch_events_into_buffer())
                .unwrap()
        };
        let process_event_buffer_thread = {
            let self_clone = Arc::clone(&main_clone);
            thread::Builder::new()
                .name("process_event_buffer_thread".to_string())
                .spawn(move || self_clone.process_event_buffer())
                .unwrap()
        };

        //let _ = heartbeat_thread.join();
        let _ = listen_for_messages_thread.join();
        let _ = _batch_events_into_buffer_thread.join();
        let _ = process_event_buffer_thread.join();
        Ok(())
    }
}
