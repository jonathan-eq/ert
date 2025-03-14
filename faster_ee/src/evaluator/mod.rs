use crate::events::dispatcher_event::checksum_event::ForwardModelStepChecksum;

use crate::events::Event;

use crossbeam::queue::SegQueue;
use ee_threads::do_heartbeat_clients::HeartBeat;
use ee_threads::DestinationHandler;
use log::{error, info};

use crate::events::snapshot_event::*;
use crate::snapshots::EnsembleSnapshot;
mod ee_threads;

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

#[derive(PartialEq)]
enum EnsembleState {
    Started,
    Stopped,
    Cancelled,
    Failed,
    Unknown,
}
impl EnsembleState {
    fn as_str(&self) -> &'static str {
        match self {
            EnsembleState::Started => "Starting",
            EnsembleState::Unknown => "Unknown",
            EnsembleState::Stopped => "Stopped",
            EnsembleState::Cancelled => "Cancelled",
            EnsembleState::Failed => "Failed",
        }
    }
}

type ZMQIdentity = Vec<u8>;
#[derive(Clone)]
pub struct EE {
    _thread: Arc<Option<std::thread::JoinHandle<()>>>,
    address: Arc<String>,
    is_running: Arc<AtomicBool>,
    _client_connected: Arc<RwLock<HashSet<ZMQIdentity>>>,
    _dispatchers_connected: Arc<RwLock<HashSet<ZMQIdentity>>>,
    _ert_identity: Arc<RwLock<Option<ZMQIdentity>>>,
    _events_to_send: Arc<SegQueue<QueueEvents>>,
    _batch_processing_queue: Arc<SegQueue<HashMap<DestinationHandler, Vec<Event>>>>,
    _events: Arc<SegQueue<Event>>,
    _router_socket: Arc<Mutex<Option<zmq::Socket>>>,
    _ensemble_id: Arc<RwLock<Option<String>>>,
    _ensemble_status: Arc<RwLock<EnsembleState>>,
    _main_snapshot: Arc<RwLock<EnsembleSnapshot>>,
    _batching_interval: Arc<Duration>,
    _max_batch_size: Arc<i64>,
    server_curve: Arc<Option<(Vec<u8>, Vec<u8>)>>,
    is_socket_ready: Arc<AtomicBool>,
}
#[derive(Debug)]
pub enum QueueEvents {
    HeartBeat(HeartBeat),
    EnsembleSnapshot(EESnapshotUpdateEvent),
    Checksum(ForwardModelStepChecksum),
}

impl EE {
    //#[new]
    //#[pyo3(signature=(address, server_curve, ensemble_id))]
    pub fn new(address: String, server_curve: Option<(Vec<u8>, Vec<u8>)>) -> Self {
        let instance = Self {
            _thread: Arc::new(None),
            address: Arc::new(address),
            is_running: Arc::new(AtomicBool::new(true)),
            _client_connected: Arc::new(RwLock::new(HashSet::new())),
            _dispatchers_connected: Arc::new(RwLock::new(HashSet::new())),
            _ert_identity: Arc::new(RwLock::new(None)),
            _events_to_send: Arc::new(SegQueue::new()),
            _router_socket: Arc::new(Mutex::new(None)),
            _ensemble_id: Arc::new(RwLock::new(None)),
            _main_snapshot: Arc::new(RwLock::new(EnsembleSnapshot::default())),
            _max_batch_size: Arc::new(500),
            _batch_processing_queue: Arc::new(SegQueue::new()),
            _events: Arc::new(SegQueue::new()),
            _batching_interval: Arc::new(Duration::from_secs(1)),
            _ensemble_status: Arc::new(RwLock::new(EnsembleState::Unknown)),
            server_curve: Arc::new(server_curve),
            is_socket_ready: Arc::new(AtomicBool::new(false)),
        };
        instance
    }

    pub fn run(self: Arc<Self>) {
        match self._run() {
            Ok(_) => {
                info!("SUCCEEDED AND FINISHED EE");
            }
            Err(inner_msg) => {
                error!("{}", inner_msg);
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
        let publisher_thread = {
            let self_clone = Arc::clone(&main_clone);
            thread::Builder::new()
                .name("process_event_buffer_thread".to_string())
                .spawn(move || self_clone._publisher())
                .unwrap()
        };
        let heartbeat_thread = {
            let clone = Arc::clone(&main_clone);
            thread::Builder::new()
                .name("heartbeat_thread".to_string())
                .spawn(move || clone._do_heartbeat_clients())
                .unwrap()
        };

        //let _ = heartbeat_thread.join();
        let _ = listen_for_messages_thread.join();
        let _ = _batch_events_into_buffer_thread.join();
        let _ = process_event_buffer_thread.join();
        let _ = publisher_thread.join();
        let _ = heartbeat_thread.join();
        Ok(())
    }
}
