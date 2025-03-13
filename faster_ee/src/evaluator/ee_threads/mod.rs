pub mod batch_events_into_buffer;
pub mod do_heartbeat_clients;
pub mod listen_for_messages;
pub mod main_server;
pub mod process_event_buffer;
pub mod publisher;

#[derive(Eq, Hash, PartialEq)]
pub enum DestinationHandler {
    FMHandler,
    EnsembleStarted,
    EnsembleSucceeded,
    EnsembleCancelled,
    EnsembleFailed,
    EESnapshotUpdate,
}
