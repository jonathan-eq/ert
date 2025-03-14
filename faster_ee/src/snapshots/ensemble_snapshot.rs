use std::collections::HashMap;

use log::debug;
use serde::{Deserialize, Serialize};

use super::fm_step_snapshot::FMStepSnapshot;

use super::realization_snapshot::{RealizationSnapshot, RealizationState};
use crate::events::dispatcher_event::fm_step_event::{
    ForwardModelStepStatus, RealForwardModelStep,
};

use crate::events::ensemble_event::{EnsembleStatus, RealEnsembleEvent};
use crate::events::ert_event::RealRealization;
use crate::events::snapshot_event::EESnapshotUpdateEvent;
use crate::events::{types::*, Event};
use crate::update_field_if_set;
use crate::utils::is_none_or_empty;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnsembleSnapshot {
    #[serde(rename = "reals")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub _realization_snapshots: HashMap<RealId, RealizationSnapshot>,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub _fm_step_snapshots: HashMap<(RealId, FmStepId), FMStepSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "status")]
    pub _ensemble_state: Option<EnsembleStatus>,
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
                debug!("AFTER MERGIN!{:#?}", self);
                return self;
            }
        };
    }
    pub fn merge_snapshot(&mut self, event: &EESnapshotUpdateEvent) {
        self.update_from(&event.snapshot);
    }
    pub fn update_fm_from_event(&mut self, event: &RealForwardModelStep) -> &mut Self {
        let mut mutate_snapshot = FMStepSnapshot::new();
        mutate_snapshot.update_from_event(event);
        self._update_fm_step(event.real_id.clone(), &mutate_snapshot);
        return self;
    }

    fn _update_fm_step(&mut self, real_id: String, mutate_snapshot: &FMStepSnapshot) {
        let fm_step_id = mutate_snapshot.index.clone().unwrap();
        let step_to_update = self
            ._fm_step_snapshots
            .entry((real_id, fm_step_id))
            .or_insert_with(FMStepSnapshot::new);
        step_to_update.update_from(mutate_snapshot)
    }

    pub fn update_real_from_event(
        &mut self,
        event: &RealRealization,
        source_snapshot: &EnsembleSnapshot,
    ) -> &mut Self {
        let source_snapshot = source_snapshot;
        let mut mutate_snapshot = RealizationSnapshot::new();
        mutate_snapshot.update_from_event(event);
        self._update_realization(event.get_real_id(), &mutate_snapshot);
        if event.status == RealizationState::Timeout {
            self._handle_realization_timeout(&mutate_snapshot, event, source_snapshot);
        }
        return self;
    }
    fn _handle_realization_timeout(
        &mut self,
        mutate_snapshot: &RealizationSnapshot,
        event: &RealRealization,
        source_snapshot: &Self,
    ) {
        let mut snapshot_to_update_from = FMStepSnapshot::new();
        snapshot_to_update_from.status = Some(ForwardModelStepStatus::Failed);
        snapshot_to_update_from.end_time = mutate_snapshot.end_time;
        snapshot_to_update_from.error = Some(String::from(
            "The run is cancelled due to reaching MAX_RUNTIME",
        ));
        for (fm_step_id, source_fm_step_snapshot) in source_snapshot
            ._realization_snapshots
            .get(&event.get_real_id())
            .and_then(|realsnapshot| Some(&realsnapshot.fm_steps))
            .unwrap_or(&HashMap::new())
        {
            if let Some(status_msg) = source_fm_step_snapshot.status.clone() {
                if status_msg != ForwardModelStepStatus::Failed {
                    let fm_idx = (event.get_real_id(), fm_step_id.clone());
                    let fm_step_snapshot = self
                        ._fm_step_snapshots
                        .entry(fm_idx)
                        .or_insert(FMStepSnapshot::new());
                    fm_step_snapshot.update_from(&snapshot_to_update_from);
                }
            }
        }
    }

    fn _update_realization(&mut self, real_id: RealId, mutate_snapshot: &RealizationSnapshot) {
        let stored_snapshot = self
            ._realization_snapshots
            .entry(real_id)
            .or_insert(RealizationSnapshot::new());
        stored_snapshot.update_from(mutate_snapshot)
    }

    fn update_ensemble_from_event(&mut self, event: &RealEnsembleEvent) -> &mut EnsembleSnapshot {
        self._ensemble_state = Some(event.state.clone());
        self
    }

    pub fn update_from(&mut self, other_snapshot: &Self) {
        update_field_if_set!(self, other_snapshot, _ensemble_state);
        for (real_id, other_real_data) in &other_snapshot._realization_snapshots {
            self._update_realization(real_id.clone(), &other_real_data);
        }
        for (real_fm_step_id, fm_step_data) in &other_snapshot._fm_step_snapshots {
            self._update_fm_step(real_fm_step_id.0.clone(), &fm_step_data);
        }
    }

    pub fn update_snapshot(&self, events: &Vec<Event>) -> EnsembleSnapshot {
        let mut snapshot_mutate_event: EnsembleSnapshot = Self::default();
        let update_snapshot = self._update_snapshot(&mut snapshot_mutate_event, &events);
        return update_snapshot.to_owned();
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

    pub fn create_new_with_synced_fm_steps_into_realizations(&self) -> Self {
        let mut self_clone = self.clone();
        for ((real_id, fm_step_id), fm_step) in &self_clone._fm_step_snapshots {
            // Ensure the realization snapshot exists
            self_clone
                ._realization_snapshots
                .entry(real_id.clone())
                .or_insert_with(|| {
                    let mut snapshot = RealizationSnapshot::new();
                    snapshot.fm_steps = HashMap::new();
                    snapshot
                });

            // Get a mutable reference to the realization snapshot
            if let Some(realization_snapshot) = self_clone._realization_snapshots.get_mut(real_id) {
                // Ensure fm_steps exists and insert the fm_step
                realization_snapshot
                    .fm_steps
                    .entry(fm_step_id.clone())
                    .or_insert_with(|| fm_step.clone());
            }
        }
        self_clone
    }

    pub fn get_successful_realizations(self) -> Vec<i64> {
        let mut completed_realizations: Vec<i64> = Vec::new();
        for (idx, snapshot) in self._realization_snapshots {
            if let Some(status) = snapshot.status {
                if status == RealizationState::Finished {
                    completed_realizations.push(idx.parse().unwrap());
                }
            }
        }
        completed_realizations
    }
}
