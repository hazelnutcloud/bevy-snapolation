use std::{time::Duration, fmt::Debug};

use bevy::{prelude::*, utils::HashMap};
use serde::{Serialize, Deserialize};

#[derive(Component, Clone)]
pub struct Vault {
    pub vault_size: usize,
    pub vault: Vec<Snapshot>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snapshot {
    pub id: u64,
    pub time: Duration,
    pub entities: SnapolationEntities
}

pub type SnapolationEntities = HashMap<String, Vec<SnapolationEntity>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StateValue {
    Number(f32),
    Degree(f32),
    Radian(f32),
    Quat(Vec4)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapolationEntity {
    pub id: u64,
    pub state: HashMap<String, StateValue>
}

impl Vault {
    pub fn get_by_id(&self, id: u64) -> Option<&Snapshot> {
        self.vault.iter().find(|snapshot| snapshot.id == id)
    }

    pub fn clear(&mut self) {
        self.vault.clear();
    }

    pub fn get_latest(&mut self) -> Option<&Snapshot> {
        self.vault.sort_unstable_by(|a, b| { b.time.cmp(&a.time) });
        self.vault.first()
    }

    pub fn get_two_closest(&self, time: Duration) -> Option<Vec<Option<Snapshot>>> {
        let mut sorted = self.vault.clone();
        sorted.sort_unstable_by(|a, b| { b.time.cmp(&a.time) });
        
        for (index, snapshot) in sorted.iter().enumerate() {
            if snapshot.time.le(&time) {
                if let Some(newer_snapshot) = sorted.get(index - 1) {
                    return Some(vec![Some(newer_snapshot.clone()), Some(snapshot.clone())]);
                } else {
                    return Some(vec![None, Some(snapshot.clone())]);
                }
            }
        }

        None
    }

    pub fn get_closest(&self, time: Duration) -> Option<Snapshot> {
        let mut sorted = self.vault.clone();
        sorted.sort_unstable_by(|a, b| { b.time.cmp(&a.time) });

        for (index, snapshot) in sorted.iter().enumerate() {
            if snapshot.time.le(&time) {
                if index == 0 { return Some(snapshot.clone()) }
                if let Some(newer_snapshot) = sorted.get(index - 1) {
                    let older = (time.as_millis() as i128 - snapshot.time.as_millis() as i128).abs();
                    let newer = (time.as_millis() as i128 - newer_snapshot.time.as_millis() as i128).abs();
                    if newer <= older {
                        return Some(newer_snapshot.clone());
                    }
                    return Some(snapshot.clone());
                } else {
                    return Some(snapshot.clone());
                }
            }
        }

        None
    }

    pub fn add(&mut self, snapshot: Snapshot) {
        self.vault.sort_unstable_by(|a, b| { b.time.cmp(&a.time) });

        if self.vault.len() >= self.vault_size {
            self.vault.pop();
        }

        self.vault.insert(0, snapshot);
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self { vault_size: 120, vault: Vec::new() }
    }
}