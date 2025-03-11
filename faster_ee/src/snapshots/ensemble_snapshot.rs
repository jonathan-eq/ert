use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::fm_step_snapshot::FMStepSnapshot;
use super::fm_step_snapshot::*;
use super::realization_snapshot::{RealizationEvent, RealizationSnapshot};
use crate::events::dispatcher_event::FMEvent;
use crate::events::ensemble_event::EnsembleEvent;
use crate::events::snapshot_event::EESnapshotUpdateEvent;
use crate::events::{types::*, Event};
use crate::update_field_if_set;

struct ForwardModelStepChecksum;

#[derive(Serialize, Clone, Debug)]
pub struct EnsembleSnapshot {
    _realization_snapshots: HashMap<RealId, RealizationSnapshot>,
    #[serde(serialize_with = "serialize_tuple_keys")]
    _fm_step_snapshots: HashMap<(RealId, FmStepId), FMStepSnapshot>,
    _ensemble_state: Option<String>,
}
// Custom function to convert tuple keys to strings
fn serialize_tuple_keys<S>(
    value: &HashMap<(FmStepId, String), FMStepSnapshot>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let map: HashMap<String, &FMStepSnapshot> = value
        .iter()
        .map(|((k1, k2), v)| (format!("({}, {})", k1, k2), v))
        .collect();

    map.serialize(serializer)
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
    pub fn merge_snapshot(&mut self, event: &EESnapshotUpdateEvent) {
        println!("TRYING TO MERGE SNAPSHOT!");
    }
    pub fn update_fm_from_event(&mut self, event: &FMEvent) -> &mut Self {
        let mut mutate_snapshot = FMStepSnapshot::new();
        mutate_snapshot.update_from_event(event);
        self._update_fm_step(event.get_real_id().clone(), &mutate_snapshot);
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
        event: &RealizationEvent,
        source_snapshot: &EnsembleSnapshot,
    ) -> &mut Self {
        let source_snapshot = source_snapshot;
        let mut mutate_snapshot = RealizationSnapshot::new();
        mutate_snapshot.update_from_event(event);
        self._update_realization(event.get_real_id(), &mutate_snapshot);
        if let RealizationEvent::RealizationTimeout(timeout_out_realization) = event {
            self._handle_realization_timeout(&mutate_snapshot, event, source_snapshot);
        }
        return self;
    }
    fn _handle_realization_timeout(
        &mut self,
        mutate_snapshot: &RealizationSnapshot,
        event: &RealizationEvent,
        source_snapshot: &Self,
    ) {
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

    fn update_ensemble_from_event(&mut self, event: &EnsembleEvent) -> &mut EnsembleSnapshot {
        self._ensemble_state = Some(String::from(event.get_status()));
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
}
