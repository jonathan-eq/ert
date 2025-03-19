use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use chrono::Utc;
use log::{debug, warn};

use crate::{
    events::{ensemble_event::EnsembleStatus, Event},
    EE,
};

use super::DestinationHandler;

impl EE {
    pub fn _batch_events_into_buffer(self: Arc<Self>) {
        while self.is_running() | !self._events.is_empty() {
            let mut batch: HashMap<DestinationHandler, Vec<Event>> = HashMap::new();
            let start_time = Utc::now();
            let mut events_in_map_count: i64 = 0;
            while (events_in_map_count < *self._max_batch_size)
                && (Utc::now() < start_time + *self._batching_interval)
            {
                events_in_map_count = events_in_map_count + 1;
                match self._events.pop() {
                    Some(event) => match event {
                        Event::FMEvent(ref inner_evt) => {
                            debug!("Adding FMEvent to batch {:#?}", inner_evt);
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(event);
                        }
                        Event::RealizationEvent(_) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(event);
                        }
                        Event::EnsembleEvent(ref inner_event) => {
                            batch
                                .entry(match inner_event.state {
                                    EnsembleStatus::Cancelled => {
                                        DestinationHandler::EnsembleCancelled
                                    }
                                    EnsembleStatus::Failed => DestinationHandler::EnsembleFailed,
                                    EnsembleStatus::Started => DestinationHandler::EnsembleStarted,
                                    EnsembleStatus::Succeeded => {
                                        DestinationHandler::EnsembleSucceeded
                                    }
                                    EnsembleStatus::Unknown => DestinationHandler::EnsembleStarted,
                                })
                                .or_default()
                                .push(event);
                        }
                        Event::EESnapshotUpdateEvent(_) => {
                            batch
                                .entry(DestinationHandler::EESnapshotUpdate)
                                .or_default()
                                .push(event);
                            warn!("We got EESnapshotUpdateEvent in batch_events_into_buffer. This should not happen");
                        }
                        Event::EEFullSnapshotEvent(_) => {
                            batch
                                .entry(DestinationHandler::EEFullSnapshot)
                                .or_default()
                                .push(event);
                            warn!("We got EEFullSnapshotEvent in buffer. This might happen.");
                        }
                    },
                    None => {
                        thread::sleep(Duration::from_millis(100));
                        events_in_map_count = events_in_map_count - 1
                    }
                }
            }
            if batch.len() > 0 {
                debug!("Adding batch of {} events to processing queue", batch.len());
                self._batch_processing_queue.push(batch);
            }
            if self._events.len() > 500 {
                warn!(
                    "There are a lot of events left in queue ({})",
                    self._events.len()
                )
            }
        }
    }
}
