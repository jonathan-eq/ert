use std::{any::type_name, sync::Arc};

use crate::{
    events::{client_event::ClientEvent, dispatcher_event::DispatcherEvent},
    EE,
};

fn print_type<T>(prefix: &str, _: &T) {
    println!("{}{}", prefix, type_name::<T>());
}

impl EE {
    pub fn _handle_event_from_dispatcher(self: &Arc<Self>, json_string: &String) {
        println!("Trying to handle event from dispatcher: {}", json_string);
        match serde_json::from_str::<DispatcherEvent>(json_string.as_str()) {
            Ok(event) => {
                print_type("event_handler got type from dispatcher", &event);
                self._events.push(event);
            }
            Err(err) => {
                eprintln!("{:#?}", err)
            }
        }
    }
    pub fn _handle_event_from_client(self: &Arc<Self>, json_string: &String) {
        match serde_json::from_str::<ClientEvent>(json_string.as_str()) {
            Ok(event) => match event {
                ClientEvent::EEUserCancel(_) => {
                    println!("Client asked to cancel.");
                    self._signal_cancel();
                }
                ClientEvent::EEUserDone(_) => {
                    println!("Client signalled done");
                    self.stop();
                }
            },
            Err(err) => {
                eprintln!("handle_client failed {:?}", err)
            }
        }
    }
}
