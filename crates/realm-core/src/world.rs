use crate::types::{MobTemplate, NpcTemplate, QuestTemplate, RoomTemplate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static MOB_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn next_mob_id() -> String {
    format!(
        "mob_{}",
        MOB_COUNTER.fetch_add(1, Ordering::Relaxed) + 1
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMob {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "templateId")]
    pub template_id: String,
    pub hp: i32,
    #[serde(rename = "maxHp")]
    pub max_hp: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elite: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LiveRoom {
    pub template: RoomTemplate,
    pub items: Vec<String>,
    pub mobs: Vec<LiveMob>,
    pub npcs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorldData {
    pub rooms: Vec<RoomTemplate>,
    pub mobs: Vec<MobTemplate>,
    pub npcs: Vec<NpcTemplate>,
    pub quests: Vec<QuestTemplate>,
}

struct WorldState {
    rooms: HashMap<String, RoomTemplate>,
    mobs: HashMap<String, MobTemplate>,
    npcs: HashMap<String, NpcTemplate>,
    quests: HashMap<String, QuestTemplate>,
    live_rooms: HashMap<String, LiveRoom>,
    respawn_pending: HashSet<String>,
    world_path: PathBuf,
}

#[derive(Clone)]
pub struct World {
    state: Arc<Mutex<WorldState>>,
}

impl World {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let contents = fs::read_to_string(&path)?;
        let data: WorldData = serde_json::from_str(&contents)?;

        let mut rooms = HashMap::new();
        for room in data.rooms {
            rooms.insert(room.id.clone(), room);
        }

        let mut mobs = HashMap::new();
        for mob in data.mobs {
            mobs.insert(mob.id.clone(), mob);
        }

        let mut npcs = HashMap::new();
        for npc in data.npcs {
            npcs.insert(npc.id.clone(), npc);
        }

        let mut quests = HashMap::new();
        for quest in data.quests {
            quests.insert(quest.id.clone(), quest);
        }

        let world = Self {
            state: Arc::new(Mutex::new(WorldState {
                rooms,
                mobs,
                npcs,
                quests,
                live_rooms: HashMap::new(),
                respawn_pending: HashSet::new(),
                world_path: path,
            })),
        };

        world.init_live_rooms();
        Ok(world)
    }

    pub fn rooms(&self) -> HashMap<String, RoomTemplate> {
        let state = self.state.lock().expect("world poisoned");
        state.rooms.clone()
    }

    pub fn mobs(&self) -> HashMap<String, MobTemplate> {
        let state = self.state.lock().expect("world poisoned");
        state.mobs.clone()
    }

    pub fn npcs(&self) -> HashMap<String, NpcTemplate> {
        let state = self.state.lock().expect("world poisoned");
        state.npcs.clone()
    }

    pub fn quests(&self) -> HashMap<String, QuestTemplate> {
        let state = self.state.lock().expect("world poisoned");
        state.quests.clone()
    }

    fn init_live_rooms(&self) {
        let mut state = self.state.lock().expect("world poisoned");
        let room_ids: Vec<String> = state.rooms.keys().cloned().collect();
        for id in room_ids {
            let template = state.rooms.get(&id).expect("room exists").clone();
            let live_mobs: Vec<LiveMob> = template
                .mobs
                .iter()
                .map(|mob_id| {
                    let tmpl = state.mobs.get(mob_id).expect("mob template missing");
                    let scale = if tmpl.elite { 1.5 } else { 1.0 };
                    let hp = (tmpl.hp as f64 * scale).floor() as i32;
                    LiveMob {
                        instance_id: next_mob_id(),
                        template_id: mob_id.clone(),
                        hp,
                        max_hp: hp,
                        elite: if tmpl.elite { Some(true) } else { None },
                    }
                })
                .collect();

            state.live_rooms.insert(
                id,
                LiveRoom {
                    template: template.clone(),
                    items: template.items.clone(),
                    mobs: live_mobs,
                    npcs: template.npcs.clone(),
                },
            );
        }
    }

    pub fn get_room(&self, room_id: &str) -> Option<LiveRoom> {
        let state = self.state.lock().expect("world poisoned");
        state.live_rooms.get(room_id).cloned()
    }

    pub fn get_room_mut<F, R>(&self, room_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut LiveRoom) -> R,
    {
        let mut state = self.state.lock().expect("world poisoned");
        state.live_rooms.get_mut(room_id).map(f)
    }

    pub fn remove_mob(&self, room_id: &str, instance_id: &str) -> Option<LiveMob> {
        let removed = {
            let mut state = self.state.lock().expect("world poisoned");
            let room = state.live_rooms.get_mut(room_id)?;
            let idx = room
                .mobs
                .iter()
                .position(|m| m.instance_id == instance_id)?;
            let removed = room.mobs.remove(idx);
            let template_id = removed.template_id.clone();
            Some((removed, template_id))
        }?;

        let (removed, template_id) = removed;
        self.schedule_respawn(room_id.to_string(), template_id);
        Some(removed)
    }

    fn schedule_respawn(&self, room_id: String, template_id: String) {
        let key = format!("{room_id}:{template_id}");
        let respawn_seconds = {
            let mut state = self.state.lock().expect("world poisoned");
            if state.respawn_pending.contains(&key) {
                return;
            }
            state.respawn_pending.insert(key.clone());
            state
                .mobs
                .get(&template_id)
                .map(|m| m.respawn_seconds)
                .unwrap_or(0)
        };

        let world = Arc::clone(&self.state);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(respawn_seconds)).await;

            let mut state = world.lock().expect("world poisoned");
            state.respawn_pending.remove(&key);

            let max_count = state
                .rooms
                .get(&room_id)
                .map(|t| t.mobs.iter().filter(|id| *id == &template_id).count())
                .unwrap_or(0);
            let count = state
                .live_rooms
                .get(&room_id)
                .map(|room| {
                    room.mobs
                        .iter()
                        .filter(|m| m.template_id == template_id)
                        .count()
                })
                .unwrap_or(0);
            let tmpl = state.mobs.get(&template_id).cloned();

            if count < max_count {
                let Some(tmpl) = tmpl else {
                    return;
                };
                let Some(room) = state.live_rooms.get_mut(&room_id) else {
                    return;
                };
                let scale = if tmpl.elite { 1.5 } else { 1.0 };
                let hp = (tmpl.hp as f64 * scale).floor() as i32;
                room.mobs.push(LiveMob {
                    instance_id: next_mob_id(),
                    template_id: template_id.clone(),
                    hp,
                    max_hp: hp,
                    elite: if tmpl.elite { Some(true) } else { None },
                });
            }
        });
    }

    pub fn get_zone(&self, room_id: &str) -> String {
        let state = self.state.lock().expect("world poisoned");
        state
            .rooms
            .get(room_id)
            .map(|r| r.zone.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    pub fn reload(&self) -> anyhow::Result<Vec<String>> {
        let path = {
            let state = self.state.lock().expect("world poisoned");
            state.world_path.clone()
        };

        let contents = fs::read_to_string(&path)?;
        let data: WorldData = serde_json::from_str(&contents)?;
        let mut logs = Vec::new();

        let mut state = self.state.lock().expect("world poisoned");
        state.rooms.clear();
        state.mobs.clear();
        state.npcs.clear();
        state.quests.clear();

        for room in data.rooms {
            state.rooms.insert(room.id.clone(), room);
        }
        for mob in data.mobs {
            state.mobs.insert(mob.id.clone(), mob);
        }
        for npc in data.npcs {
            state.npcs.insert(npc.id.clone(), npc);
        }
        for quest in data.quests {
            state.quests.insert(quest.id.clone(), quest);
        }

        for (id, template) in state.rooms.clone() {
            if let Some(existing) = state.live_rooms.get_mut(&id) {
                existing.template = template.clone();
                existing.npcs = template.npcs.clone();
                logs.push(format!("Updated room: {}", template.name));
            } else {
                let live_mobs: Vec<LiveMob> = template
                    .mobs
                    .iter()
                    .map(|mob_id| {
                        let tmpl = state.mobs.get(mob_id).expect("mob template missing");
                        LiveMob {
                            instance_id: next_mob_id(),
                            template_id: mob_id.clone(),
                            hp: tmpl.hp,
                            max_hp: tmpl.hp,
                            elite: None,
                        }
                    })
                    .collect();
                state.live_rooms.insert(
                    id,
                    LiveRoom {
                        template: template.clone(),
                        items: template.items.clone(),
                        mobs: live_mobs,
                        npcs: template.npcs.clone(),
                    },
                );
                logs.push(format!("Added room: {}", template.name));
            }
        }

        Ok(logs)
    }
}