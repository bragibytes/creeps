use std::collections::HashMap;

use crate::creep::{CreepAction, Position};
use crate::structure::StructureType;
use crate::world::WorldState;

/// Run one simulation tick. Player WASM hooks in here later (between phases).
pub fn tick_world(world: &mut WorldState) {
    world.tick += 1;

    for room in world.rooms.values_mut() {
        room.tick = world.tick;

        let sources: Vec<(String, Position, i32)> = room
            .structures
            .iter()
            .filter(|s| s.structure_type == StructureType::Source)
            .map(|s| (s.id.clone(), s.pos, s.energy.unwrap_or(0)))
            .collect();

        let spawns: HashMap<String, (String, Position)> = room
            .structures
            .iter()
            .filter(|s| s.structure_type == StructureType::Spawn)
            .filter_map(|s| s.owner.clone().map(|o| (o, (s.id.clone(), s.pos))))
            .collect();

        for creep in &mut room.creeps {
            if creep.fatigue > 0 {
                creep.fatigue = creep.fatigue.saturating_sub(1);
                continue;
            }

            match &creep.action {
                CreepAction::Idle => {
                    if let Some((source_id, source_pos, _energy)) = sources
                        .iter()
                        .filter(|(_, _, e)| *e > 0)
                        .min_by_key(|(_, pos, _)| manhattan(creep.pos, *pos))
                        .map(|(id, pos, e)| (id.clone(), *pos, *e))
                    {
                        if creep.pos == source_pos {
                            creep.action = CreepAction::Harvest { structure_id: source_id };
                        } else {
                            creep.action = CreepAction::Move {
                                x: source_pos.x,
                                y: source_pos.y,
                            };
                        }
                    }
                }
                CreepAction::Move { x, y } => {
                    let step = step_toward(creep.pos, Position::new(*x, *y));
                    creep.pos = step;
                    creep.fatigue = crate::creep::Creep::move_cost(&creep.body);
                    if creep.pos.x == *x && creep.pos.y == *y {
                        creep.action = CreepAction::Idle;
                    }
                }
                CreepAction::Harvest { structure_id } => {
                    let source_energy = room
                        .structures
                        .iter()
                        .find(|s| &s.id == structure_id)
                        .and_then(|s| s.energy)
                        .unwrap_or(0);
                    let free = creep.carrying_capacity - creep.carrying_energy;
                    if free > 0 && source_energy > 0 {
                        let take = 2.min(source_energy).min(free);
                        if let Some(source) = room
                            .structures
                            .iter_mut()
                            .find(|s| &s.id == structure_id)
                        {
                            source.energy = Some(source_energy - take);
                        }
                        creep.carrying_energy += take;
                    }

                    let source_empty = room
                        .structures
                        .iter()
                        .find(|s| &s.id == structure_id)
                        .and_then(|s| s.energy)
                        .unwrap_or(0)
                        == 0;

                    if creep.carrying_energy >= creep.carrying_capacity || source_empty {
                        if let Some((spawn_id, spawn_pos)) =
                            spawns.get(&creep.owner).cloned()
                        {
                            if creep.pos == spawn_pos {
                                let deposit = creep.carrying_energy;
                                creep.carrying_energy = 0;
                                if let Some(spawn) = room
                                    .structures
                                    .iter_mut()
                                    .find(|s| s.id == spawn_id)
                                {
                                    let cap = spawn.energy_capacity.unwrap_or(300);
                                    let cur = spawn.energy.unwrap_or(0);
                                    spawn.energy = Some((cur + deposit).min(cap));
                                }
                                creep.action = CreepAction::Idle;
                            } else {
                                creep.action = CreepAction::Move {
                                    x: spawn_pos.x,
                                    y: spawn_pos.y,
                                };
                            }
                        } else {
                            creep.action = CreepAction::Idle;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if world.tick.is_multiple_of(300) {
        for room in world.rooms.values_mut() {
            for structure in &mut room.structures {
                if structure.structure_type == StructureType::Source {
                    let cap = structure.energy_capacity.unwrap_or(3000);
                    let cur = structure.energy.unwrap_or(0);
                    structure.energy = Some((cur + crate::constants::SOURCE_ENERGY_REGEN).min(cap));
                }
            }
        }
    }
}

fn manhattan(a: Position, b: Position) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn step_toward(from: Position, to: Position) -> Position {
    let mut pos = from;
    if pos.x < to.x {
        pos.x += 1;
    } else if pos.x > to.x {
        pos.x -= 1;
    } else if pos.y < to.y {
        pos.y += 1;
    } else if pos.y > to.y {
        pos.y -= 1;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_creeps_harvest_over_ticks() {
        let mut world = WorldState::new_sector_3x3();
        world.register_player("lord_a");
        world.bootstrap_player("lord_a").unwrap();

        let room_name = crate::room::room_name_from_coords(0, 0);
        for _ in 0..200 {
            tick_world(&mut world);
        }

        let room = world.room(&room_name).unwrap();
        let spawn = room.spawn_for("lord_a").unwrap();
        assert!(
            spawn.energy.unwrap_or(0) > 0,
            "starter harvest AI should deposit energy at spawn"
        );
    }
}