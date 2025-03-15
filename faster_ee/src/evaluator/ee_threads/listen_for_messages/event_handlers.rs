use std::sync::Arc;

use log::{debug, error, info, warn};

use crate::{
    evaluator::QueueEvents,
    events::{
        client_event::ClientEvent, dispatcher_event::DispatcherEvent, ert_event::ErtEvent,
        EECancelled, Event,
    },
    EE,
};

impl EE {
    pub fn _handle_event_from_dispatcher(self: &Arc<Self>, json_string: &String) {
        match serde_json::from_str::<DispatcherEvent>(json_string.as_str()) {
            Ok(event) => match event {
                DispatcherEvent::ForwardModelStepChecksum(event) => {
                    self._events_to_send.push(QueueEvents::Checksum(event));
                }
                DispatcherEvent::EnsembleFailed(event)
                | DispatcherEvent::EnsembleStarted(event) => {
                    self._events.push(Event::EnsembleEvent(event));
                }

                DispatcherEvent::ForwardModelStepFailure(event)
                | DispatcherEvent::ForwardModelStepRunning(event)
                | DispatcherEvent::ForwardModelStepStart(event)
                | DispatcherEvent::ForwardModelStepSuccess(event) => {
                    self._events.push(Event::FMEvent(event));
                }
            },
            Err(err) => {
                error!(
                    " Couldn't deserialize event from dispatcher '{json_string}' from dispatcher {:#?}",
                    err.to_string()
                )
            }
        }
    }
    pub fn _handle_event_from_client(self: &Arc<Self>, json_string: &String) {
        match serde_json::from_str::<ClientEvent>(json_string.as_str()) {
            Ok(event) => match event {
                ClientEvent::EEUserCancel(event) => {
                    info!("Client asked to cancel.");
                    self._signal_cancel(EECancelled {
                        ensemble_id: self
                            ._ensemble_id
                            .read()
                            .unwrap()
                            .clone()
                            .unwrap_or_default(),
                        monitor: Some(event.monitor.clone()),
                    });
                }
                ClientEvent::EEUserDone(event) => {
                    info!("Client signalled done");
                    self._events_to_send.push(QueueEvents::UserDone(event));
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
                ErtEvent::RealizationFailed(event)
                | ErtEvent::RealizationPending(event)
                | ErtEvent::RealizationRunning(event)
                | ErtEvent::RealizationSuccess(event)
                | ErtEvent::RealizationTimeout(event)
                | ErtEvent::RealizationUnknown(event)
                | ErtEvent::RealizationWaiting(event) => {
                    self._events.push(Event::RealizationEvent(event));
                }
                ErtEvent::EESnapshotUpdate(event) | ErtEvent::EEFullSnapshot(event) => {
                    warn!("GOT EE SNAPSHOT FROM ERT");
                    *self._ensemble_id.write().unwrap() = Some(event.ensemble.clone());
                    self._events.push(Event::EESnapshotUpdateEvent(event));
                }
            },
            Err(err) => {
                error!(
                    " Couldn't deserialize event from ert '{json_string}' from ert {:#?}",
                    err
                )
            }
        }
    }
}
