use serde::{Deserialize, Serialize};

/// Wire messages for the Screeps-style colony game (separate from legacy MUD protocol).

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ColonyClientMessage {
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        password: String,
    },
    /// Upload compiled player logic (wasm32-unknown-unknown).
    UploadModule {
        #[serde(rename = "wasmBytes", with = "serde_bytes")]
        wasm_bytes: Vec<u8>,
    },
    /// Request full snapshot (on connect or after desync).
    RequestSnapshot,
    /// Place spawn in owned/neutral room (bootstrap handled server-side for now).
    PlaceSpawn {
        room: String,
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ColonyServerMessage {
    Snapshot {
        snapshot: ColonySnapshot,
    },
    Tick {
        tick: u64,
    },
    ScriptLog {
        text: String,
    },
    Error {
        text: String,
    },
    Prompt {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonySnapshot {
    pub tick: u64,
    pub viewer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<PlayerColonySnapshot>,
    pub rooms: Vec<RoomSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerColonySnapshot {
    pub username: String,
    pub gcl: u8,
    #[serde(rename = "cpuBucket")]
    pub cpu_bucket: u64,
    #[serde(rename = "spawnRoom", skip_serializing_if = "Option::is_none")]
    pub spawn_room: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSnapshot {
    pub name: String,
    pub tick: u64,
    pub structures: Vec<StructureSnapshot>,
    pub creeps: Vec<CreepSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSnapshot {
    pub id: String,
    #[serde(rename = "structureType")]
    pub structure_type: String,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    #[serde(rename = "maxHp")]
    pub max_hp: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<i32>,
    #[serde(rename = "energyCapacity", skip_serializing_if = "Option::is_none")]
    pub energy_capacity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreepSnapshot {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub x: i32,
    pub y: i32,
    pub action: String,
    #[serde(rename = "carryingEnergy")]
    pub carrying_energy: i32,
    #[serde(rename = "carryingCapacity")]
    pub carrying_capacity: i32,
}