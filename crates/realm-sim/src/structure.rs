use serde::{Deserialize, Serialize};

use crate::creep::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StructureType {
    Spawn,
    Extension,
    Container,
    Tower,
    Storage,
    Wall,
    Rampart,
    Road,
    Controller,
    Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: String,
    #[serde(rename = "structureType")]
    pub structure_type: StructureType,
    pub pos: Position,
    pub hp: i32,
    #[serde(rename = "maxHp")]
    pub max_hp: i32,
    /// Spawn/extension/storage energy store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i32>,
    #[serde(rename = "energyCapacity", skip_serializing_if = "Option::is_none")]
    pub energy_capacity: Option<i32>,
    pub owner: Option<String>,
}

impl Structure {
    pub fn source(id: String, pos: Position) -> Self {
        Self {
            id,
            structure_type: StructureType::Source,
            pos,
            hp: 1000,
            max_hp: 1000,
            energy: Some(super::constants::SOURCE_ENERGY_CAPACITY),
            energy_capacity: Some(super::constants::SOURCE_ENERGY_CAPACITY),
            owner: None,
        }
    }

    pub fn controller(id: String, pos: Position) -> Self {
        Self {
            id,
            structure_type: StructureType::Controller,
            pos,
            hp: 1000,
            max_hp: 1000,
            energy: None,
            energy_capacity: None,
            owner: None,
        }
    }

    pub fn spawn(id: String, pos: Position, owner: String) -> Self {
        Self {
            id,
            structure_type: StructureType::Spawn,
            pos,
            hp: 5000,
            max_hp: 5000,
            energy: Some(300),
            energy_capacity: Some(300),
            owner: Some(owner),
        }
    }
}