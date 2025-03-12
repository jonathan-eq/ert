use std::{any::type_name, sync::Arc};

use log::{debug, error, info};

use crate::{
    evaluator::QueueEvents,
    events::{client_event::ClientEvent, dispatcher_event::DispatcherEvent},
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
                DispatcherEvent::Checksum(event) => {
                    debug!("Got checksum event from dispatcher {json_string}");
                    self._events_to_send.push(QueueEvents::Checksum(event));
                }
                _ => {
                    self._events.push(event);
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
}
