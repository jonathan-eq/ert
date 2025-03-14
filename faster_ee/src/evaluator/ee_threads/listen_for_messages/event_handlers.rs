use std::{any::type_name, sync::Arc};

use log::{debug, error, info, warn};

use crate::{
    evaluator::QueueEvents,
    events::{
        client_event::ClientEvent,
        dispatcher_event::{DispatcherEvent, FMEvent},
        ensemble_event::EnsembleEvent,
        ert_event::{ErtEvent, RealizationEvent},
        Event,
    },
    EE,
};

pub fn print_type<T>(prefix: &str, _: &T) {
    debug!("{}{}", prefix, type_name::<T>());
}

pub fn get_type_name<T>(_: &T) -> &str {
    type_name::<T>()
}

impl EE {
    pub fn _handle_event_from_dispatcher(self: &Arc<Self>, json_string: &String) {
        match serde_json::from_str::<DispatcherEvent>(json_string.as_str()) {
            Ok(event) => match event {
                DispatcherEvent::ForwardModelStepChecksum(event) => {
                    self._events_to_send.push(QueueEvents::Checksum(event));
                }
                DispatcherEvent::EnsembleFailed(event) => {
                    self._events
                        .push(Event::EnsembleEvent(EnsembleEvent::EnsembleFailed(event)));
                }
                DispatcherEvent::EnsembleStarted(event) => {
                    self._events
                        .push(Event::EnsembleEvent(EnsembleEvent::EnsembleStarted(event)));
                }
                DispatcherEvent::ForwardModelStepFailure(event) => {
                    self._events
                        .push(Event::FMEvent(FMEvent::ForwardModelStepFailure(event)));
                }
                DispatcherEvent::ForwardModelStepRunning(event) => {
                    self._events
                        .push(Event::FMEvent(FMEvent::ForwardModelStepRunning(event)));
                }
                DispatcherEvent::ForwardModelStepStart(event) => {
                    self._events
                        .push(Event::FMEvent(FMEvent::ForwardModelStepStart(event)));
                }
                DispatcherEvent::ForwardModelStepSuccess(event) => {
                    self._events
                        .push(Event::FMEvent(FMEvent::ForwardModelStepSuccess(event)));
                }
            },
            Err(err) => {
                error!(
                    " Couldn't deserialize event '{json_string}' from dispatcher {:#?}",
                    err
                )
            }
        }
    }
    pub fn _handle_event_from_client(self: &Arc<Self>, json_string: &String) {
        match serde_json::from_str::<ClientEvent>(json_string.as_str()) {
            Ok(event) => match event {
                ClientEvent::EEUserCancel(_) => {
                    info!("Client asked to cancel.");
                    self._signal_cancel();
                }
                ClientEvent::EEUserDone(_) => {
                    info!("Client signalled done");
                    self.stop();
                }
            },
            Err(err) => {
                error!(
                    "Could not deserialize event from client '{json_string}' {:?}",
                    err
                )
            }
        }
    }
    pub fn _handle_event_from_ert(self: &Arc<Self>, json_string: &String) {
        debug!("HANDLING EVENT FROM ERT {}", json_string);
        match serde_json::from_str::<ErtEvent>(json_string.as_str()) {
            Ok(event) => match event {
                ErtEvent::RealizationFailed(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationFailed(event),
                    ));
                }
                ErtEvent::RealizationPending(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationPending(event),
                    ));
                }
                ErtEvent::RealizationRunning(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationRunning(event),
                    ));
                }
                ErtEvent::RealizationSuccess(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationSuccess(event),
                    ));
                }
                ErtEvent::RealizationTimeout(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationTimeout(event),
                    ));
                }
                ErtEvent::RealizationUnknown(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationUnknown(event),
                    ));
                }
                ErtEvent::RealizationWaiting(event) => {
                    self._events.push(Event::RealizationEvent(
                        RealizationEvent::RealizationWaiting(event),
                    ));
                }
                ErtEvent::EESnapshotUpdate(event) => {
                    warn!("GOT EE SNAPSHOT FROM ERT");
                    *self._ensemble_id.write().unwrap() = Some(event.ensemble.clone());
                    self._events.push(Event::EESnapshotUpdateEvent(event));
                }
            },
            Err(err) => {
                error!(
                    " Couldn't deserialize event '{json_string}' from ert {:#?}",
                    err
                )
            }
        }
    }
}
