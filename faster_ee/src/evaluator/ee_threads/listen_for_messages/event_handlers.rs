use std::sync::Arc;

use crate::{
    events::{client_event::ClientEvent, dispatcher_event::DispatcherEvent},
    EE,
};

impl EE {
    pub fn _handle_event_from_dispatcher(self: &Arc<Self>, json_string: &String) {
        println!("Trying to handle event from dispatcher: {}", json_string);
        match serde_json::from_str::<DispatcherEvent>(json_string.as_str()) {
            Ok(event) => {
                print!("Got event from dispatcher {:?}", event);
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
