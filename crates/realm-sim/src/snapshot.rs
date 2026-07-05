use realm_protocol::colony::{
    ColonySnapshot, CreepSnapshot, PlayerColonySnapshot, RoomSnapshot, StructureSnapshot,
};

use crate::structure::StructureType;
use crate::world::WorldState;

fn structure_type_name(t: StructureType) -> &'static str {
    match t {
        StructureType::Spawn => "spawn",
        StructureType::Extension => "extension",
        StructureType::Container => "container",
        StructureType::Tower => "tower",
        StructureType::Storage => "storage",
        StructureType::Wall => "wall",
        StructureType::Rampart => "rampart",
        StructureType::Road => "road",
        StructureType::Controller => "controller",
        StructureType::Source => "source",
    }
}

pub fn world_snapshot(world: &WorldState, viewer: &str) -> ColonySnapshot {
    let viewer_key = viewer.to_lowercase();
    let player = world.players.get(&viewer_key);

    let rooms: Vec<RoomSnapshot> = world
        .rooms
        .values()
        .map(|room| RoomSnapshot {
            name: room.name.0.clone(),
            tick: room.tick,
            structures: room
                .structures
                .iter()
                .map(|s| StructureSnapshot {
                    id: s.id.clone(),
                    structure_type: structure_type_name(s.structure_type).into(),
                    x: s.pos.x,
                    y: s.pos.y,
                    hp: s.hp,
                    max_hp: s.max_hp,
                    energy: s.energy,
                    energy_capacity: s.energy_capacity,
                    owner: s.owner.clone(),
                })
                .collect(),
            creeps: room
                .creeps
                .iter()
                .map(|c| CreepSnapshot {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    owner: c.owner.clone(),
                    x: c.pos.x,
                    y: c.pos.y,
                    action: format!("{:?}", c.action),
                    carrying_energy: c.carrying_energy,
                    carrying_capacity: c.carrying_capacity,
                })
                .collect(),
        })
        .collect();

    ColonySnapshot {
        tick: world.tick,
        viewer: viewer.to_string(),
        player: player.map(|p| PlayerColonySnapshot {
            username: p.username.clone(),
            gcl: p.gcl,
            cpu_bucket: p.cpu_bucket,
            spawn_room: p.spawn_room.as_ref().map(|r| r.0.clone()),
        }),
        rooms,
    }
}