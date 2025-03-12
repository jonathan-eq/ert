use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use chrono::Utc;
use log::{debug, warn};

use crate::{
    evaluator::ee_threads::listen_for_messages::event_handlers::get_type_name,
    events::{
        dispatcher_event::{DispatcherEvent, FMEvent},
        Event,
    },
    EE,
};

use super::DestinationHandler;

impl EE {
    pub fn _batch_events_into_buffer(self: Arc<Self>) {
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
                        DispatcherEvent::ForwardModelStepStart(event) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(Event::FMEvent(FMEvent::ForwardModelStepStart(event)));
                        }
                        DispatcherEvent::ForwardModelStepRunning(event) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(Event::FMEvent(FMEvent::ForwardModelStepRunning(event)));
                        }
                        DispatcherEvent::ForwardModelStepSuccess(event) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(Event::FMEvent(FMEvent::ForwardModelStepSuccess(event)));
                        }
                        DispatcherEvent::ForwardModelStepFailure(event) => {
                            batch
                                .entry(DestinationHandler::FMHandler)
                                .or_default()
                                .push(Event::FMEvent(FMEvent::ForwardModelStepFailure(event)));
                        }
                        _ => {
                            warn!("Not handling this type of event yet {:?}", inner_event);
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
