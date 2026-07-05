use serde::{Deserialize, Serialize};

use crate::constants::ROOM_SIZE;
use crate::creep::{Creep, Position};
use crate::structure::{Structure, StructureType};

/// Screeps-style room name, e.g. `W1N1`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomName(pub String);

impl RoomName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomState {
    pub name: RoomName,
    pub structures: Vec<Structure>,
    pub creeps: Vec<Creep>,
    pub tick: u64,
}

impl RoomState {
    pub fn new_empty(name: impl Into<String>) -> Self {
        Self {
            name: RoomName::new(name),
            structures: Vec::new(),
            creeps: Vec::new(),
            tick: 0,
        }
    }

    /// Procedural neutral room: controller + two sources, no owner. No NPC creeps.
    pub fn new_sector(name: impl Into<String>) -> Self {
        let name = RoomName::new(name);
        let controller_pos = Position::new(ROOM_SIZE / 2, ROOM_SIZE / 2);
        let source_a = Position::new(ROOM_SIZE / 4, ROOM_SIZE / 4);
        let source_b = Position::new(ROOM_SIZE * 3 / 4, ROOM_SIZE * 3 / 4);

        let structures = vec![
            Structure::controller(format!("{}_controller", name.0), controller_pos),
            Structure::source(format!("{}_source_0", name.0), source_a),
            Structure::source(format!("{}_source_1", name.0), source_b),
        ];

        Self {
            name,
            structures,
            creeps: Vec::new(),
            tick: 0,
        }
    }

    pub fn controller(&self) -> Option<&Structure> {
        self.structures
            .iter()
            .find(|s| s.structure_type == StructureType::Controller)
    }

    pub fn controller_mut(&mut self) -> Option<&mut Structure> {
        self.structures
            .iter_mut()
            .find(|s| s.structure_type == StructureType::Controller)
    }

    pub fn sources(&self) -> impl Iterator<Item = &Structure> {
        self.structures
            .iter()
            .filter(|s| s.structure_type == StructureType::Source)
    }

    pub fn spawn_for(&self, owner: &str) -> Option<&Structure> {
        self.structures.iter().find(|s| {
            s.structure_type == StructureType::Spawn && s.owner.as_deref() == Some(owner)
        })
    }

    pub fn is_owned_by(&self, owner: &str) -> bool {
        self.controller()
            .and_then(|c| c.owner.as_deref())
            .is_some_and(|o| o == owner)
    }

    pub fn world_coords(&self) -> (i32, i32) {
        parse_room_coords(&self.name.0).unwrap_or((0, 0))
    }

    pub fn set_world_coords_hint(&self) -> (i32, i32) {
        self.world_coords()
    }
}

/// Parse Screeps-style `1W1N` → (-1, 1).
pub fn parse_room_coords(name: &str) -> Option<(i32, i32)> {
    let upper = name.to_uppercase();
    let ew_idx = upper.find(['E', 'W'])?;
    let ns_idx = upper.find(['N', 'S'])?;
    if ns_idx <= ew_idx + 1 {
        return None;
    }

    let x_mag: i32 = upper[..ew_idx].parse().ok()?;
    let y_mag: i32 = upper[ew_idx + 1..ns_idx].parse().ok()?;
    let x = if upper.as_bytes()[ew_idx] == b'W' {
        -x_mag
    } else {
        x_mag
    };
    let y = if upper.as_bytes()[ns_idx] == b'S' {
        -y_mag
    } else {
        y_mag
    };
    Some((x, y))
}

pub fn room_name_from_coords(x: i32, y: i32) -> String {
    let ew = if x < 0 { 'W' } else { 'E' };
    let ns = if y < 0 { 'S' } else { 'N' };
    format!("{}{}{}{}", x.unsigned_abs(), ew, y.unsigned_abs(), ns)
}