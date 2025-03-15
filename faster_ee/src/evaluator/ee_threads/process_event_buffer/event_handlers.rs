use std::sync::Arc;

use log::debug;

use crate::{
    evaluator::{EnsembleState, QueueEvents},
    events::{
        snapshot_event::{EESnapshotEvent, EESnapshotUpdateEvent},
        EECancelled, Event,
    },
    snapshots::{fm_step_snapshot::FMStepSnapshot, EnsembleSnapshot},
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
                self._signal_cancel(EECancelled {
                    ensemble_id: self
                        ._ensemble_id
                        .read()
                        .unwrap()
                        .clone()
                        .unwrap_or_default(),
                    monitor: None,
                });
            }
        }
    }
    pub fn _signal_cancel(self: &Arc<Self>, cancelled_event: EECancelled) {
        self._events_to_send
            .push(QueueEvents::UserCancelledEE(cancelled_event));
        self.stop()
    }
    pub fn _stopped_handler(self: &Arc<Self>, events: &Vec<Event>) {
        if *self._ensemble_status.read().unwrap() == EnsembleState::Failed {
            return;
        }
        self._send_run_statistics_to_ert();
        self._create_update_snapshot_and_apply_to_main_snapshot(events);
    }
    fn _send_run_statistics_to_ert(self: &Arc<Self>) {
        let mut max_memory_usage = -1;
        let mut overspent_cpu_msgs: Vec<String> = Vec::new();
        for ((real_id, _), fm_snapshot) in self
            ._main_snapshot
            .read()
            .unwrap()
            .clone()
            ._fm_step_snapshots
        {
            max_memory_usage = std::cmp::max(
                max_memory_usage,
                fm_snapshot.max_memory_usage.unwrap_or_default(),
            );
            if let Some(error_msg) = self.detect_overspent_cpu(4, &real_id, &fm_snapshot) {
                overspent_cpu_msgs.push(error_msg);
            }
        }
    }
    fn detect_overspent_cpu(
        self: &Arc<Self>,
        num_cpu: i64,
        real_id: &String,
        fm_step: &FMStepSnapshot,
    ) -> Option<String> {
        Some(String::from("yes"))
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
        let event = EESnapshotEvent::new(
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
