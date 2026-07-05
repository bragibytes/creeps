//! Realm colony client — local sim demo until server tick + WASM are wired.
//!
//! Run: `cargo run -p realm-game`

use bevy::prelude::*;
use realm_protocol::colony::ColonySnapshot;
use realm_sim::{snapshot::world_snapshot, tick::tick_world, world::WorldState, ROOM_SIZE};

const TILE_SIZE: f32 = 20.0;
const TICK_INTERVAL_SECS: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creeps".into(),
                resolution: (ROOM_SIZE as f32 * TILE_SIZE + 200.0, ROOM_SIZE as f32 * TILE_SIZE + 80.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<SimResource>()
        .add_systems(Startup, (setup_camera, setup_grid, bootstrap_local_sim))
        .add_systems(Update, (advance_sim, sync_entities, update_hud))
        .run();
}

#[derive(Resource)]
struct SimResource {
    world: WorldState,
    snapshot: ColonySnapshot,
    active_room: String,
    tick_accum: f32,
}

impl Default for SimResource {
    fn default() -> Self {
        let world = WorldState::new_sector_3x3();
        let active_room = realm_sim::room::room_name_from_coords(0, 0);
        let snapshot = world_snapshot(&world, "local");
        Self {
            world,
            snapshot,
            active_room,
            tick_accum: 0.0,
        }
    }
}

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct CreepVisual {
    creep_id: String,
}

#[derive(Component)]
struct StructureVisual {
    structure_id: String,
}

#[derive(Component)]
struct HudText;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_grid(mut commands: Commands) {
    let half = ROOM_SIZE as f32 * TILE_SIZE / 2.0;
    for x in 0..ROOM_SIZE {
        for y in 0..ROOM_SIZE {
            let dark = (x + y) % 2 == 0;
            let color = if dark {
                Color::srgb(0.12, 0.14, 0.12)
            } else {
                Color::srgb(0.15, 0.17, 0.15)
            };
            commands.spawn((
                Tile,
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * TILE_SIZE - half + TILE_SIZE / 2.0,
                    y as f32 * TILE_SIZE - half + TILE_SIZE / 2.0,
                    0.0,
                ),
            ));
        }
    }
}

fn bootstrap_local_sim(mut sim: ResMut<SimResource>) {
    sim.world.register_player("local");
    sim.world.bootstrap_player("local").expect("bootstrap local lord");
    sim.snapshot = world_snapshot(&sim.world, "local");
}

fn advance_sim(time: Res<Time>, mut sim: ResMut<SimResource>) {
    sim.tick_accum += time.delta_secs();
    if sim.tick_accum >= TICK_INTERVAL_SECS {
        sim.tick_accum -= TICK_INTERVAL_SECS;
        tick_world(&mut sim.world);
        sim.snapshot = world_snapshot(&sim.world, "local");
    }
}

fn tile_to_world(x: i32, y: i32) -> Vec3 {
    let half = ROOM_SIZE as f32 * TILE_SIZE / 2.0;
    Vec3::new(
        x as f32 * TILE_SIZE - half + TILE_SIZE / 2.0,
        y as f32 * TILE_SIZE - half + TILE_SIZE / 2.0,
        1.0,
    )
}

fn sync_entities(
    mut commands: Commands,
    sim: Res<SimResource>,
    creeps: Query<(Entity, &CreepVisual)>,
    structures: Query<(Entity, &StructureVisual)>,
) {
    let room = sim
        .snapshot
        .rooms
        .iter()
        .find(|r| r.name == sim.active_room);
    let Some(room) = room else {
        return;
    };

    for (entity, visual) in &creeps {
        if !room.creeps.iter().any(|c| c.id == visual.creep_id) {
            commands.entity(entity).despawn();
        }
    }
    for (entity, visual) in &structures {
        if !room.structures.iter().any(|s| s.id == visual.structure_id) {
            commands.entity(entity).despawn();
        }
    }

    let existing_creeps: Vec<_> = creeps.iter().map(|(e, v)| (v.creep_id.clone(), e)).collect();
    for creep in &room.creeps {
        if let Some((_, entity)) = existing_creeps.iter().find(|(id, _)| id == &creep.id) {
            commands
                .entity(*entity)
                .insert(Transform::from_translation(tile_to_world(creep.x, creep.y)));
        } else {
            let color = if creep.owner == "local" {
                Color::srgb(0.3, 0.7, 1.0)
            } else {
                Color::srgb(1.0, 0.35, 0.35)
            };
            commands.spawn((
                CreepVisual {
                    creep_id: creep.id.clone(),
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.7)),
                    ..default()
                },
                Transform::from_translation(tile_to_world(creep.x, creep.y)),
            ));
        }
    }

    let existing_structures: Vec<_> = structures
        .iter()
        .map(|(e, v)| (v.structure_id.clone(), e))
        .collect();
    for structure in &room.structures {
        let color = match structure.structure_type.as_str() {
            "source" => Color::srgb(1.0, 0.85, 0.1),
            "controller" => Color::srgb(0.6, 0.4, 0.9),
            "spawn" => Color::srgb(0.9, 0.5, 0.2),
            _ => Color::srgb(0.5, 0.5, 0.5),
        };
        let size = if structure.structure_type == "source" {
            TILE_SIZE * 0.9
        } else {
            TILE_SIZE * 0.85
        };
        if let Some((_, entity)) = existing_structures
            .iter()
            .find(|(id, _)| id == &structure.id)
        {
            commands
                .entity(*entity)
                .insert(Transform::from_translation(tile_to_world(structure.x, structure.y)));
        } else {
            commands.spawn((
                StructureVisual {
                    structure_id: structure.id.clone(),
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(tile_to_world(structure.x, structure.y)),
            ));
        }
    }
}

fn update_hud(mut commands: Commands, sim: Res<SimResource>, hud: Query<Entity, With<HudText>>) {
    let room = sim
        .snapshot
        .rooms
        .iter()
        .find(|r| r.name == sim.active_room);
    let spawn_energy = room
        .and_then(|r| {
            r.structures
                .iter()
                .find(|s| s.structure_type == "spawn" && s.owner.as_deref() == Some("local"))
        })
        .and_then(|s| s.energy)
        .unwrap_or(0);

    let text = format!(
        "Creeps (local demo)\nRoom {} | Tick {} | Spawn energy {} | Creeps {}\nSandbox PvP only — no NPCs",
        sim.active_room,
        sim.snapshot.tick,
        spawn_energy,
        room.map(|r| r.creeps.len()).unwrap_or(0),
    );

    if let Ok(entity) = hud.single() {
        commands.entity(entity).insert(Text2d::new(text));
    } else {
        commands.spawn((
            HudText,
            Text2d::new(text),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            Transform::from_xyz(0.0, ROOM_SIZE as f32 * TILE_SIZE / 2.0 + 30.0, 10.0),
        ));
    }
}