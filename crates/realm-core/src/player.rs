use crate::db::StoredPlayer;
use crate::guilds::find_guild_by_member;
use crate::items::ITEMS;
use crate::quests::{QuestProgress, QuestStatus};
use crate::types::{class_stats, xp_for_level};
use crate::world::World;
use realm_protocol::PlayerSnapshot;
use std::collections::HashSet;

#[derive(Clone)]
pub struct PlayerSession {
    pub data: StoredPlayer,
    pub combat_target: Option<String>,
    pub pvp_target: Option<String>,
    pub in_duel: bool,
    pub party_leader: Option<String>,
    pub searched_rooms: HashSet<String>,
    pub authenticated: bool,
}

impl PlayerSession {
    pub fn new(data: StoredPlayer) -> Self {
        Self {
            data,
            combat_target: None,
            pvp_target: None,
            in_duel: false,
            party_leader: None,
            searched_rooms: HashSet::new(),
            authenticated: false,
        }
    }

    pub fn username(&self) -> &str {
        &self.data.username
    }

    pub fn room_id(&self) -> &str {
        &self.data.room_id
    }

    pub fn in_combat(&self) -> bool {
        self.combat_target.is_some() || self.pvp_target.is_some()
    }

    pub fn clear_combat(&mut self) {
        self.combat_target = None;
        self.pvp_target = None;
    }

    pub fn total_attack(&self) -> i32 {
        let cls = class_stats(self.data.class_name);
        let mut atk = cls.attack + (self.data.level as i32 - 1) * 2;
        if let Some(weapon_id) = &self.data.equipment.weapon {
            if let Some(weapon) = ITEMS.get(weapon_id) {
                if let Some(attack) = weapon.attack {
                    atk += attack;
                }
            }
        }
        atk
    }

    pub fn total_defense(&self) -> i32 {
        let cls = class_stats(self.data.class_name);
        let mut def = cls.defense + (self.data.level as i32 - 1);
        if let Some(armor_id) = &self.data.equipment.armor {
            if let Some(armor) = ITEMS.get(armor_id) {
                if let Some(defense) = armor.defense {
                    def += defense;
                }
            }
        }
        def
    }

    pub fn xp_to_level(&self) -> i32 {
        xp_for_level(self.data.level)
    }

    pub fn add_xp(&mut self, amount: i32) -> Vec<String> {
        let mut messages = Vec::new();
        self.data.xp += amount;
        messages.push(format!("You gain {amount} experience."));

        while self.data.xp >= self.xp_to_level() {
            self.data.xp -= self.xp_to_level();
            self.level_up(&mut messages);
        }
        messages
    }

    fn level_up(&mut self, messages: &mut Vec<String>) {
        self.data.level += 1;
        let cls = class_stats(self.data.class_name);
        let hp_gain = (cls.max_hp as f64 * 0.15).floor() as i32 + 5;
        let mp_gain = (cls.max_mp as f64 * 0.12).floor() as i32 + 3;
        self.data.max_hp += hp_gain;
        self.data.max_mp += mp_gain;
        self.data.hp = self.data.max_hp;
        self.data.mp = self.data.max_mp;
        messages.push(format!(
            "*** LEVEL UP! You are now level {}! ***",
            self.data.level
        ));
        messages.push(format!("HP +{hp_gain}, MP +{mp_gain}"));
    }

    pub fn add_item(&mut self, item_id: &str) {
        self.data.inventory.push(item_id.to_string());
    }

    pub fn remove_item(&mut self, item_id: &str) -> bool {
        if let Some(idx) = self
            .data
            .inventory
            .iter()
            .position(|id| id == item_id)
        {
            self.data.inventory.remove(idx);
            true
        } else {
            false
        }
    }

    pub fn count_item(&self, item_id: &str) -> i32 {
        self.data
            .inventory
            .iter()
            .filter(|id| id.as_str() == item_id)
            .count() as i32
    }

    pub fn get_active_quests(&self) -> Vec<&QuestProgress> {
        self.data
            .quests
            .iter()
            .filter(|q| q.status == QuestStatus::Active)
            .collect()
    }

    pub fn to_snapshot(&self, world: &World) -> PlayerSnapshot {
        let rooms = world.rooms();
        let room = rooms.get(&self.data.room_id);
        PlayerSnapshot {
            username: self.data.username.clone(),
            class_name: self.data.class_name,
            level: self.data.level,
            hp: self.data.hp,
            max_hp: self.data.max_hp,
            mp: self.data.mp,
            max_mp: self.data.max_mp,
            xp: self.data.xp,
            xp_to_level: self.xp_to_level(),
            gold: self.data.gold,
            room: self.data.room_id.clone(),
            room_name: room.map(|r| r.name.clone()),
            zone: room.map(|r| r.zone.clone()),
            in_combat: if self.in_combat() { Some(true) } else { None },
            party_leader: self.party_leader.clone(),
            in_duel: if self.in_duel { Some(true) } else { None },
            title: self.data.title.clone(),
            guild_name: find_guild_by_member(&self.data.username).map(|g| g.name),
        }
    }
}