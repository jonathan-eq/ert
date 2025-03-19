pub mod event_handlers;
use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use super::DestinationHandler;
use crate::{events::Event, EE};

impl EE {
    pub fn process_event_buffer(self: Arc<Self>) {
        while self.is_running() | !self._batch_processing_queue.is_empty() {
            match self._batch_processing_queue.pop() {
                Some(batch) => {
                    self._handle_batch(&batch);
                }
                None => {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }

    fn _handle_batch(self: &Arc<Self>, batch: &HashMap<DestinationHandler, Vec<Event>>) {
        for (handler, events) in batch {
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
                DestinationHandler::EESnapshotUpdate => {
                    self._update_snapshot_handler(&events);
                }
                DestinationHandler::EEFullSnapshot => {
                    self._overwrite_snapshot_handler(&events);
                }
            };
        }
    }
}
