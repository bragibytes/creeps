use crate::items::{format_item_name, ITEMS};
use crate::player::PlayerSession;
use crate::quests::{check_quest_complete, QuestStatus};
use crate::types::class_stats;
use crate::world::{LiveMob, World};
use rand::Rng;
use std::collections::HashSet;
use std::sync::LazyLock;

pub static SAFE_ZONES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["Eldermoor"]));

pub fn is_safe_zone(world: &World, room_id: &str) -> bool {
    SAFE_ZONES.contains(world.get_zone(room_id).as_str())
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub messages: Vec<String>,
    pub player_died: bool,
    pub mob_killed: bool,
}

#[derive(Debug, Clone)]
pub struct PvpRoundResult {
    pub attacker_messages: Vec<String>,
    pub defender_messages: Vec<String>,
    pub defender_died: bool,
    pub attacker_died: bool,
}

struct ResolvedAbility {
    name: &'static str,
    damage: i32,
    cost: i32,
    locked: bool,
    level: u32,
}

fn roll_damage(attack: i32, defense: i32) -> i32 {
    let base = (attack - defense / 2).max(1);
    let variance = (base as f64 * 0.3).floor() as i32;
    let mut rng = rand::thread_rng();
    base + rng.gen_range(0..=variance)
}

fn resolve_ability(player: &PlayerSession, slot: u8) -> ResolvedAbility {
    let cls = class_stats(player.data.class_name);
    if slot == 2 {
        if player.data.level < cls.ability2_level {
            return ResolvedAbility {
                name: cls.ability2,
                damage: 0,
                cost: 0,
                locked: true,
                level: cls.ability2_level,
            };
        }
        return ResolvedAbility {
            name: cls.ability2,
            damage: cls.ability2_damage,
            cost: cls.ability2_cost,
            locked: false,
            level: cls.ability2_level,
        };
    }
    ResolvedAbility {
        name: cls.ability,
        damage: cls.ability_damage,
        cost: cls.ability_cost,
        locked: false,
        level: 1,
    }
}

pub fn player_attack(
    player: &mut PlayerSession,
    mob: &mut LiveMob,
    world: &World,
    party_peers: &mut [&mut PlayerSession],
) -> CombatResult {
    let tmpl = world
        .mobs()
        .get(&mob.template_id)
        .expect("mob template missing")
        .clone();
    let mut messages = Vec::new();
    let dmg = roll_damage(player.total_attack(), tmpl.defense);
    mob.hp -= dmg;
    messages.push(format!(
        "You strike {} for {dmg} damage! ({}/{}) HP)",
        tmpl.name, mob.hp, mob.max_hp
    ));

    if mob.hp <= 0 {
        return handle_mob_death(player, mob, world, messages, party_peers);
    }

    let counter_dmg = roll_damage(tmpl.attack, player.total_defense());
    player.data.hp -= counter_dmg;
    messages.push(format!(
        "{} hits you for {counter_dmg} damage! ({}/{}) HP)",
        tmpl.name, player.data.hp, player.data.max_hp
    ));

    if player.data.hp <= 0 {
        player.data.hp = 0;
        return CombatResult {
            messages,
            player_died: true,
            mob_killed: false,
        };
    }

    CombatResult {
        messages,
        player_died: false,
        mob_killed: false,
    }
}

pub fn player_ability(
    player: &mut PlayerSession,
    mob: &mut LiveMob,
    world: &World,
    slot: u8,
) -> CombatResult {
    let ability = resolve_ability(player, slot);
    let mut messages = Vec::new();

    if ability.locked {
        messages.push(format!("Ability unlocks at level {}.", ability.level));
        return CombatResult {
            messages,
            player_died: false,
            mob_killed: false,
        };
    }

    if player.data.mp < ability.cost {
        messages.push(format!(
            "Not enough MP! {} costs {} MP.",
            ability.name, ability.cost
        ));
        return CombatResult {
            messages,
            player_died: false,
            mob_killed: false,
        };
    }

    player.data.mp -= ability.cost;
    let tmpl = world
        .mobs()
        .get(&mob.template_id)
        .expect("mob template missing")
        .clone();
    let dmg = roll_damage(ability.damage + player.total_attack(), tmpl.defense);
    mob.hp -= dmg;
    messages.push(format!(
        "*** {}! *** You deal {dmg} damage to {}! ({}/{}) HP)",
        ability.name, tmpl.name, mob.hp, mob.max_hp
    ));

    if mob.hp <= 0 {
        return handle_mob_death(player, mob, world, messages, &mut []);
    }

    let counter_dmg = roll_damage(tmpl.attack, player.total_defense());
    player.data.hp -= counter_dmg;
    messages.push(format!(
        "{} retaliates for {counter_dmg} damage! ({}/{}) HP)",
        tmpl.name, player.data.hp, player.data.max_hp
    ));

    if player.data.hp <= 0 {
        player.data.hp = 0;
        return CombatResult {
            messages,
            player_died: true,
            mob_killed: false,
        };
    }

    CombatResult {
        messages,
        player_died: false,
        mob_killed: false,
    }
}

pub fn player_ability_vs_player(
    attacker: &mut PlayerSession,
    defender: &mut PlayerSession,
    slot: u8,
) -> PvpRoundResult {
    let ability = resolve_ability(attacker, slot);
    let mut attacker_messages = Vec::new();
    let mut defender_messages = Vec::new();

    if ability.locked {
        return PvpRoundResult {
            attacker_messages: vec![format!("Ability unlocks at level {}.", ability.level)],
            defender_messages: Vec::new(),
            defender_died: false,
            attacker_died: false,
        };
    }
    if attacker.data.mp < ability.cost {
        return PvpRoundResult {
            attacker_messages: vec![format!(
                "Not enough MP! {} costs {} MP.",
                ability.name, ability.cost
            )],
            defender_messages: Vec::new(),
            defender_died: false,
            attacker_died: false,
        };
    }

    attacker.data.mp -= ability.cost;
    let dmg = roll_damage(
        ability.damage + attacker.total_attack(),
        defender.total_defense(),
    );
    defender.data.hp -= dmg;
    attacker_messages.push(format!(
        "*** {}! *** You deal {dmg} damage to {}! ({}/{}) HP)",
        ability.name, defender.username(), defender.data.hp, defender.data.max_hp
    ));
    defender_messages.push(format!(
        "*** {} uses {}! *** You take {dmg} damage! ({}/{}) HP)",
        attacker.username(),
        ability.name,
        defender.data.hp,
        defender.data.max_hp
    ));

    if defender.data.hp <= 0 {
        defender.data.hp = 0;
        return PvpRoundResult {
            attacker_messages,
            defender_messages,
            defender_died: true,
            attacker_died: false,
        };
    }

    let counter_dmg = roll_damage(defender.total_attack(), attacker.total_defense());
    attacker.data.hp -= counter_dmg;
    attacker_messages.push(format!(
        "{} retaliates for {counter_dmg} damage! ({}/{}) HP)",
        defender.username(),
        attacker.data.hp,
        attacker.data.max_hp
    ));
    defender_messages.push(format!(
        "You retaliate for {counter_dmg} damage! ({}/{}) HP)",
        attacker.data.hp,
        attacker.data.max_hp
    ));

    if attacker.data.hp <= 0 {
        attacker.data.hp = 0;
        return PvpRoundResult {
            attacker_messages,
            defender_messages,
            defender_died: false,
            attacker_died: true,
        };
    }

    PvpRoundResult {
        attacker_messages,
        defender_messages,
        defender_died: false,
        attacker_died: false,
    }
}

fn handle_mob_death(
    player: &mut PlayerSession,
    mob: &LiveMob,
    world: &World,
    mut messages: Vec<String>,
    party_peers: &mut [&mut PlayerSession],
) -> CombatResult {
    let tmpl = world
        .mobs()
        .get(&mob.template_id)
        .expect("mob template missing")
        .clone();
    world.remove_mob(player.room_id(), &mob.instance_id);
    player.clear_combat();

    let elite_tag = if mob.elite.unwrap_or(false) || tmpl.elite {
        " ***ELITE***"
    } else {
        ""
    };
    let boss_tag = if tmpl.boss { " ***BOSS***" } else { "" };
    messages.push(format!(
        "*** {}{}{} has been slain! ***",
        tmpl.name, elite_tag, boss_tag
    ));
    player.data.kills += 1;
    if mob.template_id.contains("goblin") {
        player.data.goblin_kills += 1;
    }

    let xp_mult = if mob.elite.unwrap_or(false) || tmpl.elite {
        1.5
    } else if tmpl.boss {
        2.0
    } else {
        1.0
    };
    let gold_range = tmpl.gold.max - tmpl.gold.min + 1;
    let mut rng = rand::thread_rng();
    let gold = if gold_range > 0 {
        tmpl.gold.min + rng.gen_range(0..gold_range)
    } else {
        tmpl.gold.min
    };
    if gold > 0 {
        player.data.gold += gold;
        messages.push(format!("You loot {gold} gold."));
    }

    let xp_gain = (tmpl.xp as f64 * xp_mult).floor() as i32;
    messages.extend(player.add_xp(xp_gain));

    if !party_peers.is_empty() {
        let share = tmpl.xp / party_peers.len() as i32;
        for peer in party_peers.iter_mut() {
            if peer.username() == player.username() {
                continue;
            }
            let _ = peer.add_xp(share);
            messages.push(format!("[Party] {} gains {share} XP.", peer.username()));
        }
    }

    for drop in &tmpl.loot {
        if rng.gen::<f64>() < drop.chance {
            player.add_item(&drop.item_id);
            let _ = ITEMS.get(&drop.item_id);
            messages.push(format!("You found: {}", format_item_name(&drop.item_id)));
        }
    }

    let active_quest_ids: Vec<String> = player
        .get_active_quests()
        .into_iter()
        .map(|q| q.quest_id.clone())
        .collect();

    for quest_id in active_quest_ids {
        let Some(quest) = world.quests().get(&quest_id).cloned() else {
            continue;
        };
        let collect_counts: Vec<(String, i32)> = quest
            .objectives
            .iter()
            .filter(|obj| obj.objective_type == "collect")
            .map(|obj| (obj.target.clone(), player.count_item(&obj.target)))
            .collect();

        if let Some(qp) = player
            .data
            .quests
            .iter_mut()
            .find(|q| q.quest_id == quest_id && q.status == QuestStatus::Active)
        {
            for obj in &quest.objectives {
                if obj.objective_type == "kill" && obj.target == mob.template_id {
                    *qp.progress.entry(obj.target.clone()).or_insert(0) += 1;
                }
            }
            for (target, count) in collect_counts {
                qp.progress.insert(target, count);
            }
            if check_quest_complete(&quest, qp) {
                qp.status = QuestStatus::Complete;
                messages.push(format!(
                    "Quest complete: {}! Return to the quest giver.",
                    quest.name
                ));
            }
        }
    }

    CombatResult {
        messages,
        player_died: false,
        mob_killed: true,
    }
}

pub fn player_attack_player(
    attacker: &mut PlayerSession,
    defender: &mut PlayerSession,
) -> PvpRoundResult {
    let mut attacker_messages = Vec::new();
    let mut defender_messages = Vec::new();

    let dmg = roll_damage(attacker.total_attack(), defender.total_defense());
    defender.data.hp -= dmg;
    attacker_messages.push(format!(
        "You strike {} for {dmg} damage! ({}/{}) HP)",
        defender.username(),
        defender.data.hp,
        defender.data.max_hp
    ));
    defender_messages.push(format!(
        "{} strikes you for {dmg} damage! ({}/{}) HP)",
        attacker.username(),
        defender.data.hp,
        defender.data.max_hp
    ));

    if defender.data.hp <= 0 {
        defender.data.hp = 0;
        return PvpRoundResult {
            attacker_messages,
            defender_messages,
            defender_died: true,
            attacker_died: false,
        };
    }

    let counter_dmg = roll_damage(defender.total_attack(), attacker.total_defense());
    attacker.data.hp -= counter_dmg;
    attacker_messages.push(format!(
        "{} hits you for {counter_dmg} damage! ({}/{}) HP)",
        defender.username(),
        attacker.data.hp,
        attacker.data.max_hp
    ));
    defender_messages.push(format!(
        "You hit {} for {counter_dmg} damage! ({}/{}) HP)",
        attacker.username(),
        attacker.data.hp,
        attacker.data.max_hp
    ));

    if attacker.data.hp <= 0 {
        attacker.data.hp = 0;
        return PvpRoundResult {
            attacker_messages,
            defender_messages,
            defender_died: false,
            attacker_died: true,
        };
    }

    PvpRoundResult {
        attacker_messages,
        defender_messages,
        defender_died: false,
        attacker_died: false,
    }
}

pub fn clear_pvp_between(a: &mut PlayerSession, b: Option<&mut PlayerSession>) {
    a.clear_combat();
    if let Some(b) = b {
        b.clear_combat();
    }
}

pub fn pvp_victory_xp(victim_level: u32) -> i32 {
    (victim_level as i32 * 15).max(10)
}

pub fn player_death(player: &mut PlayerSession, killer: Option<&str>) -> Vec<String> {
    handle_player_death(player, killer)
}

pub fn handle_player_death(player: &mut PlayerSession, killer: Option<&str>) -> Vec<String> {
    player.data.deaths += 1;
    player.clear_combat();
    player.data.hp = (player.data.max_hp as f64 * 0.5).floor() as i32;
    player.data.gold = (player.data.gold - (player.data.gold as f64 * 0.1).floor() as i32).max(0);
    player.data.room_id = "eldermoor_square".into();

    vec![
        if let Some(killer) = killer {
            format!("*** You have been slain by {killer}! ***")
        } else {
            "*** You have been defeated! ***".into()
        },
        "You awaken in Eldermoor Town Square, battered but alive.".into(),
        format!("You lost some gold. HP restored to {}.", player.data.hp),
    ]
}

pub fn player_attack_pvp(
    attacker: &mut PlayerSession,
    defender: &mut PlayerSession,
) -> PvpRoundResult {
    player_attack_player(attacker, defender)
}

pub fn player_cast(
    player: &mut PlayerSession,
    mob: &mut LiveMob,
    world: &World,
) -> CombatResult {
    player_ability(player, mob, world, 1)
}

pub fn player_use_ability(
    player: &mut PlayerSession,
    mob: &mut LiveMob,
    world: &World,
) -> CombatResult {
    player_ability(player, mob, world, 1)
}

pub fn player_use_ability2(
    player: &mut PlayerSession,
    mob: &mut LiveMob,
    world: &World,
) -> CombatResult {
    player_ability(player, mob, world, 2)
}

pub fn flee_combat(player: &mut PlayerSession) -> Vec<String> {
    player.clear_combat();
    vec!["You flee from combat!".into()]
}