pub mod event_handlers;
use std::{sync::Arc, thread, time::Duration};

use crate::EE;

use super::DestinationHandler;

impl EE {
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
}
