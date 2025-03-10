use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::fm_step_snapshot::FMStepSnapshot;
use super::fm_step_snapshot::*;
use super::realization_snapshot::{RealizationEvent, RealizationSnapshot};
use crate::events::snapshots::fm_step_snapshot;
use crate::events::types::*;
use crate::update_field_if_set;

struct EESnapshotUpdateEvent;

struct ForwardModelStepChecksum;

struct EnsembleStarted;
struct EnsembleSucceeded;
struct EnsembleFailed;
struct EnsembleCancelled;

pub enum Event {
    EnsembleEvent(EnsembleEvent),
    FMEvent(FMEvent),
    RealizationEvent(RealizationEvent),
    EESnapshotUpdateEvent(EESnapshotUpdateEvent),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum DispatcherEvent {
    #[serde(rename = "forward_model.start")]
    ForwardModelStepStart(ForwardModelStepStart),
    #[serde(rename = "forward_model.running")]
    ForwardModelStepRunning(ForwardModelStepRunning),
    #[serde(rename = "forward_model.success")]
    ForwardModelStepSuccess(ForwardModelStepSuccess),
    #[serde(rename = "forward_model.failure")]
    ForwardModelStepFailure(ForwardModelStepFailure),
}

#[derive(Debug, Deserialize)]
pub struct EEUserDone {
    pub monitor: String,
    pub time: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct EEUserCancel {
    pub monitor: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type")]
pub enum ClientEvent {
    #[serde(rename = "ee.user_cancel")]
    EEUserCancel(EEUserCancel),
    #[serde(rename = "ee.user_done")]
    EEUserDone(EEUserDone),
}

pub enum EnsembleEvent {
    EnsembleStarted(EnsembleStarted),
    EnsembleSucceeded(EnsembleSucceeded),
    EnsembleFailed(EnsembleFailed),
    EnsembleCancelled(EnsembleCancelled),
}

impl EnsembleEvent {
    pub fn get_status(&self) -> &'static str {
        match self {
            EnsembleEvent::EnsembleStarted(_) => "Starting",
            EnsembleEvent::EnsembleSucceeded(_) => "Stopped",
            EnsembleEvent::EnsembleFailed(_) => "Cancelled",
            EnsembleEvent::EnsembleCancelled(_) => "Failed",
        }
    }
}
#[derive(Serialize, Clone)]
pub struct EnsembleSnapshot {
    _realization_snapshots: HashMap<RealId, RealizationSnapshot>,
    _fm_step_snapshots: HashMap<(RealId, FmStepId), FMStepSnapshot>,
    _ensemble_state: Option<String>,
}

impl Default for EnsembleSnapshot {
    fn default() -> Self {
        EnsembleSnapshot {
            _realization_snapshots: HashMap::new(),
            _fm_step_snapshots: HashMap::new(),
            _ensemble_state: None,
        }
    }
}

impl EnsembleSnapshot {
    pub fn update_from_event(
        &mut self,
        event: &Event,
        source_snapshot: Option<&EnsembleSnapshot>,
    ) -> &mut EnsembleSnapshot {
        match event {
            Event::EnsembleEvent(inner_event) => {
                self.update_ensemble_from_event(inner_event);
                return self;
            }
            Event::RealizationEvent(inner_event) => {
                self.update_real_from_event(inner_event, source_snapshot.unwrap());
                return self;
            }
            Event::FMEvent(inner_event) => return self.update_fm_from_event(inner_event),
            Event::EESnapshotUpdateEvent(inner_event) => {
                self.merge_snapshot(inner_event);
                return self;
            }
        };
    }
    pub fn merge_snapshot(&mut self, event: &EESnapshotUpdateEvent) {}
    pub fn update_fm_from_event(&mut self, event: &FMEvent) -> &mut Self {
        let mut mutate_snapshot = FMStepSnapshot::new();
        mutate_snapshot.status = Some(String::from(event.get_status()));
        mutate_snapshot.index = Some(event.get_fm_step_id());
        match event {
            FMEvent::ForwardModelStepStart(inner_event) => {
                mutate_snapshot.start_time = Some(inner_event.time);
                mutate_snapshot.stdout = inner_event.stdout.clone();
                mutate_snapshot.stderr = inner_event.stderr.clone();
            }
            FMEvent::ForwardModelStepRunning(inner_event) => {
                mutate_snapshot.current_memory_usage = inner_event.current_memory_usage;
                mutate_snapshot.max_memory_usage = inner_event.max_memory_usage;
                mutate_snapshot.cpu_seconds = inner_event.cpu_seconds;
            }
            FMEvent::ForwardModelStepSuccess(event) => {
                mutate_snapshot.end_time = Some(event.end_time);
                mutate_snapshot.error = Some(String::new());
            }
            FMEvent::ForwardModelStepFailure(event) => {
                mutate_snapshot.end_time = Some(event.end_time);
                mutate_snapshot.error = event.error.clone();
            }
        }
        self._update_fm_step(event.get_real_id(), mutate_snapshot);
        return self;
    }

    fn _update_fm_step(&mut self, real_id: String, mutate_snapshot: FMStepSnapshot) {
        let step_to_update = self
            ._fm_step_snapshots
            .entry((real_id, mutate_snapshot.index.clone().unwrap()))
            .or_insert_with(FMStepSnapshot::new);
        step_to_update.update_from(mutate_snapshot)
    }

    pub fn update_real_from_event(
        &mut self,
        event: &RealizationEvent,
        source_snapshot: &EnsembleSnapshot,
    ) -> &mut Self {
        let source_snapshot = source_snapshot;
        let mut mutate_snapshot = RealizationSnapshot::new();
        mutate_snapshot.exec_hosts = event.get_exec_hosts();
        mutate_snapshot.status = Some(String::from(event.get_status()));

        match event {
            RealizationEvent::RealizationRunning(inner_event) => {
                mutate_snapshot.start_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationFailed(inner_event) => {
                mutate_snapshot.message = inner_event.message.clone();
                mutate_snapshot.end_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationSuccess(inner_event) => {
                mutate_snapshot.end_time = Some(inner_event.time);
            }
            RealizationEvent::RealizationTimeout(inner_event) => {
                mutate_snapshot.end_time = Some(inner_event.time);
            }
            _ => {}
        }
        self._update_realization(event.get_real_id(), mutate_snapshot.clone());
        if let RealizationEvent::RealizationTimeout(_) = event {
            let mut snapshot_to_update_from = FMStepSnapshot::new();
            snapshot_to_update_from.status = Some(String::from("Failed"));
            snapshot_to_update_from.end_time = mutate_snapshot.end_time;
            snapshot_to_update_from.error = Some(String::from(
                "The run is cancelled due to reaching MAX_RUNTIME",
            ));
            for (fm_step_id, source_fm_step_snapshot) in source_snapshot
                ._realization_snapshots
                .get(&event.get_real_id())
                .and_then(|realsnapshot| realsnapshot.fm_steps.as_ref())
                .unwrap_or(&HashMap::new())
            {
                if let Some(status_msg) = &source_fm_step_snapshot.status {
                    if status_msg != "finished" {
                        let fm_idx = (event.get_real_id(), fm_step_id.clone());
                        let fm_step_snapshot = self
                            ._fm_step_snapshots
                            .entry(fm_idx)
                            .or_insert(FMStepSnapshot::new());
                        fm_step_snapshot.update_from(snapshot_to_update_from.clone());
                    }
                }
            }
        }
        return self;
    }
    fn _update_realization(&mut self, real_id: RealId, mutate_snapshot: RealizationSnapshot) {
        let stored_snapshot = self
            ._realization_snapshots
            .entry(real_id)
            .or_insert(RealizationSnapshot::new());
        stored_snapshot.update_from(mutate_snapshot)
    }

    fn update_ensemble_from_event(&mut self, event: &EnsembleEvent) -> &mut EnsembleSnapshot {
        self._ensemble_state = Some(String::from(event.get_status()));
        self
    }

    pub fn update_from(&mut self, other_snapshot: Self) {
        update_field_if_set!(self, other_snapshot, _ensemble_state);
        for (real_id, other_real_data) in other_snapshot._realization_snapshots {
            self._update_realization(real_id, other_real_data);
        }
        for (real_fm_step_id, fm_step_data) in other_snapshot._fm_step_snapshots {
            self._update_fm_step(real_fm_step_id.0, fm_step_data);
        }
    }

    pub fn update_snapshot(&self, events: &Vec<Event>) -> EnsembleSnapshot {
        let mut snapshot_mutate_event: EnsembleSnapshot = Self::default();
        let output_snapshot = self._update_snapshot(&mut snapshot_mutate_event, &events);
        return output_snapshot.to_owned();
    }
    pub fn _update_snapshot<'a>(
        &self,
        snapshot: &'a mut EnsembleSnapshot,
        events: &Vec<Event>,
    ) -> &'a mut EnsembleSnapshot {
        let mut loop_snapshot = snapshot;
        for event in events {
            loop_snapshot = loop_snapshot.update_from_event(&event, Some(&self));
        }
        loop_snapshot
    }
}
