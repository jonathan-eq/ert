use std::sync::Arc;

use crate::{
    evaluator::QueueEvents,
    events::{snapshot_event::EESnapshotUpdateEvent, Event},
    snapshots::EnsembleSnapshot,
};

use super::EE;

impl EE {
    pub fn _started_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() != "Failed" {
            let update_snapshot_event = self._main_snapshot.read().unwrap().update_snapshot(events);
            self._append_message(update_snapshot_event);
        }
    }
    pub fn _failed_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if (self._ensemble_status.to_string() == "Failed")
            | (self._ensemble_status.to_string() == "Cancelled")
        {
            return;
        }
        let snapshot_update_event = self._main_snapshot.read().unwrap().update_snapshot(events);
        self._append_message(snapshot_update_event);
        self._signal_cancel();
    }
    pub fn _signal_cancel(self: &Arc<Self>) {
        eprintln!("SIGNAL_CANCEL NOT IMPLEMENTED YET!");
    }
    pub fn _stopped_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() == "Failed" {
            return;
        }
        let snapshot_update_event = self._main_snapshot.read().unwrap().update_snapshot(events);
        self._append_message(snapshot_update_event);
    }
    pub fn _cancelled_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if self._ensemble_status.to_string() != "FAILED" {
            let update_snapshot_event =
                self._main_snapshot.read().unwrap().update_snapshot(&events);
            self._append_message(update_snapshot_event);
            self.stop();
        }
    }
    pub fn _fm_handler(self: &Arc<Self>, events: &Vec<Event>) {
        let update_snapshot_event = self._main_snapshot.read().unwrap().update_snapshot(&events);
        self._append_message(update_snapshot_event);
    }
    fn _append_message(self: &Arc<Self>, snapshot_update_event: EnsembleSnapshot) {
        let event = EESnapshotUpdateEvent {
            snapshot: snapshot_update_event,
            ensemble: self._ensemble_id.to_string().clone(),
        };
        self._events_to_send
            .push(QueueEvents::EnsembleSnapshot(event));
    }
}
