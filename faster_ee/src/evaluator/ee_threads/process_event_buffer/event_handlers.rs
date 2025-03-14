use std::sync::Arc;

use log::{debug, error};

use crate::{
    evaluator::{EnsembleState, QueueEvents},
    events::{snapshot_event::EESnapshotUpdateEvent, Event},
    snapshots::EnsembleSnapshot,
};

use super::EE;

impl EE {
    pub fn _started_handler(self: &Arc<Self>, events: &Vec<Event>) {
        let ensemble_status = self._ensemble_status.read().unwrap();
        if let Event::EnsembleEvent(ensemble_started_event) = events.first().clone().unwrap() {
            let _ = self
                ._ensemble_id
                .write()
                .unwrap()
                .replace(ensemble_started_event.ensemble_id.clone());
        }
        if *ensemble_status != EnsembleState::Failed {
            self._create_update_snapshot_and_apply_to_main_snapshot(events);
        }
    }

    pub fn _failed_handler(self: &Arc<Self>, events: &Vec<Event>) {
        match *self._ensemble_status.read().unwrap() {
            EnsembleState::Failed | EnsembleState::Cancelled => {}
            _ => {
                self._create_update_snapshot_and_apply_to_main_snapshot(events);
                self._signal_cancel();
            }
        }
    }
    pub fn _signal_cancel(self: &Arc<Self>) {
        error!("SIGNAL_CANCEL NOT IMPLEMENTED YET!");
    }
    pub fn _stopped_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if *self._ensemble_status.read().unwrap() == EnsembleState::Failed {
            return;
        }
        self._create_update_snapshot_and_apply_to_main_snapshot(events);
    }
    pub fn _cancelled_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if !(*self._ensemble_status.read().unwrap() != EnsembleState::Failed) {
            self._create_update_snapshot_and_apply_to_main_snapshot(events);
            self.stop();
        }
    }
    pub fn _fm_handler(self: &Arc<Self>, events: &Vec<Event>) {
        self._create_update_snapshot_and_apply_to_main_snapshot(events);
    }
    pub fn _update_snapshot_handler(self: &Arc<Self>, events: &Vec<Event>) {
        for event in events {
            if let Event::EESnapshotUpdateEvent(ee_snapshot_update_event) = event {
                self._main_snapshot
                    .write()
                    .unwrap()
                    .update_from(&ee_snapshot_update_event.snapshot);
            }
        }
        debug!("after merging{:#?}", self._main_snapshot.read().unwrap());
        self._append_message(self._main_snapshot.read().unwrap().clone()); // SHOULD BE FULL SNAPSHOT EVENT!
    }
    pub fn _create_update_snapshot_and_apply_to_main_snapshot(
        self: &Arc<Self>,
        events: &Vec<Event>,
    ) {
        let update_snapshot_event = self._main_snapshot.read().unwrap().update_snapshot(events);
        self._main_snapshot
            .write()
            .unwrap()
            .update_from(&update_snapshot_event);
        let synced_snapshot =
            update_snapshot_event.create_new_with_synced_fm_steps_into_realizations(); // I am not sure why this is done after updating the main snapshot...
        self._append_message(synced_snapshot);
    }

    fn _append_message(self: &Arc<Self>, snapshot_update_event: EnsembleSnapshot) {
        let event = EESnapshotUpdateEvent::new(
            snapshot_update_event,
            self._ensemble_id
                .read()
                .unwrap()
                .clone()
                .unwrap_or_default(),
        );
        self._events_to_send
            .push(QueueEvents::EnsembleSnapshot(event));
    }
}
