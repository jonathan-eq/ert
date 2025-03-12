#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use chrono::{DateTime, Utc};
    use std::collections::BTreeMap;

    fn compare(left: &EnsembleSnapshot, right: &EnsembleSnapshot) -> bool {
        left._realization_snapshots
            .iter()
            .all(|(key, value)| right._realization_snapshots.get(key) == Some(value))
            && left._fm_step_snapshots == right._fm_step_snapshots
    }
    use crate::{
        events::{
            dispatcher_event::{fm_step_event::ForwardModelStepStart, FMEvent},
            types::RealId,
            Event,
        },
        snapshots::{
            fm_step_snapshot::FMStepSnapshot, realization_snapshot::RealizationSnapshot,
            EnsembleSnapshot,
        },
        EE,
    };
    fn create_stub_ee() -> Arc<EE> {
        Arc::from(EE::new(
            "stub_address".to_string(),
            None,
            "stub_ensemble".to_string(),
        ))
    }
    #[ignore]
    #[test]
    fn test_create_update_snapshot_and_apply_to_main_snapshot_updates_source_snapshot() {
        let ee = create_stub_ee();
        let start_time = Utc::now();
        let mut expected_snapshot = EnsembleSnapshot {
            _ensemble_state: None,
            _fm_step_snapshots: HashMap::new(),
            _realization_snapshots: HashMap::new(),
        };
        let _fm_step_snapshot = FMStepSnapshot {
            cpu_seconds: None,
            status: Some("Pending".to_string()),
            current_memory_usage: None,
            end_time: None,
            start_time: Some(start_time),
            index: None,
            max_memory_usage: None,
            name: None,
            error: None,
            stderr: None,
            stdout: None,
        };
        expected_snapshot._fm_step_snapshots.extend((0..2).map(|n| {
            ((n.to_string(), "0".to_string()), {
                let mut my_instance = _fm_step_snapshot.clone();
                my_instance.index = Some("0".to_string());
                my_instance
            })
        }));
        expected_snapshot
            ._realization_snapshots
            .extend((0..2).map(|n| {
                (n.to_string(), {
                    {
                        let mut my_map = HashMap::new();
                        let mut fm_step = FMStepSnapshot::new();
                        fm_step.index = Some("0".to_string());
                        my_map.insert("0".to_string(), fm_step);
                        let instance = RealizationSnapshot {
                            active: None,
                            end_time: None,
                            start_time: None,
                            exec_hosts: None,
                            status: None,
                            message: None,
                            fm_steps: my_map,
                        };

                        instance
                    }
                })
            }));
        let events = Vec::from_iter((0..2).map(|n| {
            let fm = ForwardModelStepStart {
                time: start_time,
                fm_step: "0".to_string(),
                real_id: n.to_string(),
                stderr: None,
                stdout: None,
            };
            Event::FMEvent(FMEvent::ForwardModelStepStart(fm))
        }));
        ee._create_update_snapshot_and_apply_to_main_snapshot(&events);
        let actual_snapshot = ee._main_snapshot.read().unwrap().sync_before_serialize();
        assert!(compare(&actual_snapshot, &expected_snapshot));
    }
}
