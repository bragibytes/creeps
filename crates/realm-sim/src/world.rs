use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::creep::{BodyPart, BodySegment, Creep, CreepAction, Position};
use crate::room::{room_name_from_coords, RoomName, RoomState};
use crate::structure::Structure;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub username: String,
    pub gcl: u8,
    pub cpu_bucket: u64,
    pub spawn_room: Option<RoomName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub tick: u64,
    pub rooms: HashMap<String, RoomState>,
    pub players: HashMap<String, PlayerState>,
}

impl WorldState {
    pub fn new_sector_3x3() -> Self {
        let mut rooms = HashMap::new();
        for x in -1..=1 {
            for y in -1..=1 {
                let name = room_name_from_coords(x, y);
                rooms.insert(name.clone(), RoomState::new_sector(&name));
            }
        }
        Self {
            tick: 0,
            rooms,
            players: HashMap::new(),
        }
    }

    pub fn register_player(&mut self, username: impl Into<String>) -> &PlayerState {
        let username = username.into();
        let key = username.to_lowercase();
        self.players.entry(key.clone()).or_insert_with(|| PlayerState {
            username,
            gcl: crate::constants::STARTING_GCL,
            cpu_bucket: crate::constants::BASE_CPU_PER_TICK * 10,
            spawn_room: None,
        });
        self.players.get(&key).expect("player just inserted")
    }

    /// Bootstrap a new player: claim center room, place spawn, seed starter creeps.
    pub fn bootstrap_player(&mut self, username: &str) -> Result<(), String> {
        let key = username.to_lowercase();
        if self.players.get(&key).and_then(|p| p.spawn_room.clone()).is_some() {
            return Err("Player already has a spawn.".into());
        }

        let room_name = room_name_from_coords(0, 0);
        let room = self
            .rooms
            .get_mut(&room_name)
            .ok_or_else(|| format!("Missing room {room_name}"))?;

        if room.is_owned_by(username) {
            return Err("Room already claimed.".into());
        }

        if let Some(controller) = room.controller_mut() {
            controller.owner = Some(username.to_string());
        }

        let spawn_pos = Position::new(crate::constants::ROOM_SIZE / 2 + 2, crate::constants::ROOM_SIZE / 2);
        room.structures.push(Structure::spawn(
            format!("{room_name}_spawn_{username}"),
            spawn_pos,
            username.to_string(),
        ));

        let starter_body = vec![
            BodySegment {
                part: BodyPart::Work,
                hits: 1,
            },
            BodySegment {
                part: BodyPart::Carry,
                hits: 1,
            },
            BodySegment {
                part: BodyPart::Move,
                hits: 1,
            },
        ];

        for i in 0..3 {
            let offset = i as i32;
            room.creeps.push(Creep {
                id: format!("{username}_creep_{i}"),
                name: format!("{username}_{i}"),
                owner: username.to_string(),
                room: room_name.clone(),
                pos: Position::new(spawn_pos.x + offset, spawn_pos.y),
                body: starter_body.clone(),
                fatigue: 0,
                action: CreepAction::Idle,
                carrying_energy: 0,
                carrying_capacity: Creep::carrying_capacity(&starter_body),
            });
        }

        if let Some(player) = self.players.get_mut(&key) {
            player.spawn_room = Some(RoomName::new(&room_name));
        }

        Ok(())
    }

    pub fn room(&self, name: &str) -> Option<&RoomState> {
        self.rooms.get(name)
    }

    pub fn room_mut(&mut self, name: &str) -> Option<&mut RoomState> {
        self.rooms.get_mut(name)
    }
}