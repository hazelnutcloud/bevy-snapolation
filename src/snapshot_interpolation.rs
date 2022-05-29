use std::{
    f32::consts::PI,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::utils::HashMap;

use crate::vault::{Entities, SnapolationEntity, Snapshot, StateValue, Vault};

pub struct SnapshotInterpolation {
    vault: Vault,
    interpolation_buffer: Duration,
    time_offset: i128,
    server_time: Duration,
    autocorrect_time_offset: bool,
}

#[allow(dead_code)]
pub struct InterpolatedSnapshot {
	entities: Vec<SnapolationEntity>,
	percentage: f32,
	newer_id: u64,
	older_id: u64
}

impl SnapshotInterpolation {
    pub fn new(server_fps: f32) -> SnapshotInterpolation {
        SnapshotInterpolation {
            vault: Vault::default(),
            interpolation_buffer: Duration::from_secs_f32((1. / server_fps) * 3.),
            time_offset: -1,
            autocorrect_time_offset: true,
            server_time: Duration::from_secs(0),
        }
    }

    pub fn create_snapshot(entities: Entities) -> Snapshot {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Snapshot {
            id: now.as_millis() as u64,
            time: now,
            entities,
        }
    }

    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        if self.time_offset == -1 {
            self.time_offset = (now.as_millis() - snapshot.time.as_millis()) as i128;
        }

        if self.autocorrect_time_offset {
            let time_offset = (now.as_millis() - snapshot.time.as_millis()) as i128;
            let time_difference = (self.time_offset - time_offset).abs();
            if time_difference > 50 {
                self.time_offset = time_difference
            }
        }

        self.vault.add(snapshot);
    }

    pub fn interpolate(
        &mut self,
        snapshot_a: &Snapshot,
        snapshot_b: &Snapshot,
        time: Duration,
        entity_key: &str,
        state_keys: Vec<String>,
    ) -> InterpolatedSnapshot {
        let (newer, older) = match snapshot_a.time.cmp(&snapshot_b.time) {
            std::cmp::Ordering::Less => (snapshot_b, snapshot_a),
            std::cmp::Ordering::Equal => (snapshot_a, snapshot_b),
            std::cmp::Ordering::Greater => (snapshot_a, snapshot_b),
        };

        let t0 = newer.time;
        let t1 = older.time;
        let tn = time;

        let zero_percent = tn - t1;
        let hundred_percent = t0 - t1;
        let percent = zero_percent.div_duration_f32(hundred_percent);

        self.server_time =
            Duration::from_millis(time_lerp(t1.as_millis(), t0.as_millis(), percent) as u64);

        let mut interpolated_entities = Vec::new();

        if let Some(entities) = newer.entities.get(entity_key) {
            for entity in entities {
                if let Some(older_entities) = older.entities.get(entity_key) {
                    if let Some(older_entity) = older_entities.iter().find(|e| e.id == entity.id) {
                        for state_key in state_keys.iter() {
                            if let Some(state_value) = entity.state.get(state_key) {
                                if let Some(older_state_value) = older_entity.state.get(state_key) {
                                    let mut interpolated_entity = SnapolationEntity {
                                        id: entity.id,
                                        state: HashMap::new(),
                                    };

                                    match (state_value, older_state_value) {
                                        (
                                            StateValue::Number(number),
                                            StateValue::Number(older_number),
                                        ) => {
                                            interpolated_entity.state.insert(
                                                state_key.clone(),
                                                StateValue::Number(lerp(
                                                    *older_number,
                                                    *number,
                                                    percent,
                                                )),
                                            );
                                        }
                                        (
                                            StateValue::Degree(degree),
                                            StateValue::Degree(older_degree),
                                        ) => {
                                            interpolated_entity.state.insert(
                                                state_key.clone(),
                                                StateValue::Degree(degree_lerp(
                                                    *older_degree,
                                                    *degree,
                                                    percent,
                                                )),
                                            );
                                        }
                                        (
                                            StateValue::Radian(radian),
                                            StateValue::Radian(older_radian),
                                        ) => {
                                            interpolated_entity.state.insert(
                                                state_key.clone(),
                                                StateValue::Radian(radian_lerp(
                                                    *older_radian,
                                                    *radian,
                                                    percent,
                                                )),
                                            );
                                        }
                                        (StateValue::Quat(quat), StateValue::Quat(older_quat)) => {
                                            interpolated_entity.state.insert(
                                                state_key.clone(),
                                                StateValue::Quat(
                                                    older_quat.lerp(*quat, percent),
                                                ),
                                            );
                                        }
                                        _ => panic!("non-matching state value!"),
                                    }

                                    interpolated_entities.push(interpolated_entity);
                                }
                            }
                        }
                    }
                }
            }
        }

		InterpolatedSnapshot {
			entities: interpolated_entities,
			newer_id: newer.id,
			older_id: older.id,
			percentage: percent
		}
    }

	pub fn calc_interpolation(&mut self, entity_key: &str, state_keys: Vec<String>) -> Option<InterpolatedSnapshot> {
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
		let server_time = now.as_millis() as i128 - self.time_offset - self.interpolation_buffer.as_millis() as i128;

		if let Some(shots) = self.vault.get_two_closest(Duration::from_millis(server_time as u64)) {
			if let Some(newer) = shots.first().unwrap() {
				if let Some(older) = shots.last().unwrap() {
					return Some(self.interpolate(newer, older, Duration::from_millis(server_time as u64), entity_key, state_keys));
				}
			}
		}
		None
	}
}

fn time_lerp(start: u128, end: u128, t: f32) -> u128 {
    ((end - start) as f32 * t) as u128 + start
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    (end - start) * t + start
}

#[allow(unused_assignments)]
fn degree_lerp(start: f32, mut end: f32, t: f32) -> f32 {
    let mut result = 0.;
    let diff = end - start;

    if diff < -180. {
        end += 360.;
        result = lerp(start, end, t);
        if result >= 360. {
            result -= 360.;
        }
    } else if diff > 180. {
        end -= 360.;
        result = lerp(start, end, t);
        if result < 0. {
            result += 360.;
        }
    } else {
        result = lerp(start, end, t);
    }

    result
}

#[allow(unused_assignments)]
fn radian_lerp(start: f32, mut end: f32, t: f32) -> f32 {
    let mut result = 0.;
    let diff = end - start;

    if diff < -PI {
        end += PI * 2.;
        result = lerp(start, end, t);
        if result >= PI * 2. {
            result -= PI * 2.;
			return result;
        }
    } else if diff > PI {
        end -= PI * 2.;
        result = lerp(start, end, t);
        if result < 0. {
            result += PI * 2.;
			return result;
        }
    } else {
        result = lerp(start, end, t);
		return result;
    }

    result
}
