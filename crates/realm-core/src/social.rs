use std::collections::HashMap;

use realm_protocol::{OutputStyle, ServerMessage};

use crate::admin::is_admin;
use crate::duel::DuelManager;
use crate::guilds::{
    create_guild, find_guild_by_member, find_guild_by_name, invite_to_guild, is_leader,
    leave_guild,
};
use crate::items::ITEMS;
use crate::party::PartyManager;
use crate::player::PlayerSession;
use crate::trade::{TradeManager, TradeSession};
use crate::types::CRAFT_RECIPES;
use crate::world::{LiveMob, World};

fn send_output<F>(send: &mut F, username_key: &str, text: String, style: OutputStyle)
where
    F: FnMut(&str, ServerMessage),
{
    send(
        username_key,
        ServerMessage::Output {
            text,
            style: Some(style),
        },
    );
}

pub fn sync_player_meta(
    player: &mut PlayerSession,
    party: &PartyManager,
    duel: &DuelManager,
    players: &HashMap<String, PlayerSession>,
) {
    let leader_key = party.get_leader_key(player);
    player.party_leader = leader_key
        .as_ref()
        .and_then(|k| players.get(k).map(|p| p.username().to_string()));
    player.in_duel = duel.is_in_duel(player.username());
}

pub fn handle_party_command<F, R>(
    player_key: &str,
    args: &[&str],
    party: &mut PartyManager,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
    room_notify: &mut R,
) where
    F: FnMut(&str, ServerMessage),
    R: FnMut(&str, &str, Option<&str>),
{
    let Some(player) = players.get(player_key) else {
        return;
    };
    let self_username = player.username().to_string();

    let sub = args.first().map(|s| s.to_ascii_lowercase());
    let rest = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    if sub.as_deref() == Some("invite") && !rest.is_empty() {
        let Some(target_key) = find_player_key(players, &self_username, &rest) else {
            send_output(
                send,
                player_key,
                format!("No player \"{rest}\" found."),
                OutputStyle::Party,
            );
            return;
        };
        let err = {
            let leader = players.get(player_key).expect("player exists");
            let target = players.get(&target_key).expect("target exists");
            party.invite(leader, target)
        };
        if let Some(err) = err {
            send_output(send, player_key, err, OutputStyle::Party);
        } else {
            let target_username = players
                .get(&target_key)
                .expect("target exists")
                .username()
                .to_string();
            send_output(
                send,
                player_key,
                format!("Invited {target_username} to your party."),
                OutputStyle::Party,
            );
            send_output(
                send,
                &target_key,
                format!(
                    "{} invites you to a party. Type 'party join {}'.",
                    self_username, self_username
                ),
                OutputStyle::Party,
            );
        }
        return;
    }

    if sub.as_deref() == Some("join") && !rest.is_empty() {
        let err = {
            let player = players.get(player_key).expect("player exists");
            party.join(player, &rest)
        };
        if let Some(err) = err {
            send_output(send, player_key, err, OutputStyle::Party);
        } else {
            send_output(
                send,
                player_key,
                "You joined the party!".into(),
                OutputStyle::Party,
            );
            let room_id = players
                .get(player_key)
                .expect("player exists")
                .room_id()
                .to_string();
            room_notify(
                &room_id,
                &format!("{self_username} joins the party."),
                None,
            );
        }
        return;
    }

    if sub.as_deref() == Some("leave") {
        if let Some(player) = players.get(player_key) {
            party.leave(player);
        }
        send_output(
            send,
            player_key,
            "You left the party.".into(),
            OutputStyle::Party,
        );
        return;
    }

    if sub.as_deref() == Some("say") && !rest.is_empty() {
        let leader_key = {
            let player = players.get(player_key).expect("player exists");
            party
                .get_leader_key(player)
                .unwrap_or_else(|| player_key.to_string())
        };
        let names = party.get_member_usernames(&leader_key, players);
        for name in names {
            let key = name.to_lowercase();
            if players.contains_key(&key) {
                send_output(
                    send,
                    &key,
                    format!("[Party] {self_username}: {rest}"),
                    OutputStyle::Party,
                );
            }
        }
        return;
    }

    let leader_key = {
        let player = players.get(player_key).expect("player exists");
        party.get_leader_key(player)
    };
    let Some(leader_key) = leader_key else {
        send_output(
            send,
            player_key,
            "You are not in a party. Use party invite <player>.".into(),
            OutputStyle::Party,
        );
        return;
    };
    let members = party.get_member_usernames(&leader_key, players);
    send_output(
        send,
        player_key,
        format!("Party: {}", members.join(", ")),
        OutputStyle::Party,
    );
}

pub fn handle_trade_command<F>(
    player_key: &str,
    args: &[&str],
    trade: &mut TradeManager,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
) where
    F: FnMut(&str, ServerMessage),
{
    let Some(player) = players.get(player_key) else {
        return;
    };
    let self_username = player.username().to_string();
    let key = player_key.to_string();

    let sub = args.first().map(|s| s.to_ascii_lowercase());
    let rest = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    if sub.as_deref() == Some("cancel") {
        trade.cancel(&key);
        send_output(send, player_key, "Trade cancelled.".into(), OutputStyle::Trade);
        return;
    }

    if sub.as_deref() == Some("accept") && !rest.is_empty() {
        match trade.accept(&self_username, &rest) {
            Err(msg) => send_output(send, player_key, msg, OutputStyle::Trade),
            Ok(session) => {
                let partner_key = trade.get_partner(&session, &key);
                send_output(
                    send,
                    player_key,
                    "Trade session started!".into(),
                    OutputStyle::Trade,
                );
                if players.contains_key(&partner_key) {
                    send_output(
                        send,
                        &partner_key,
                        format!("{self_username} accepted your trade."),
                        OutputStyle::Trade,
                    );
                }
            }
        }
        return;
    }

    if sub.as_deref() == Some("offer") {
        if trade.get_session(&key).is_none() {
            send_output(send, player_key, "Not in a trade.".into(), OutputStyle::Trade);
            return;
        }
        if args.get(1) == Some(&"gold") {
            if let Some(amount_str) = args.get(2) {
                let amount: i32 = amount_str.parse().unwrap_or(0);
                let gold = players.get(player_key).map(|p| p.data.gold).unwrap_or(0);
                if amount <= 0 || amount > gold {
                    send_output(
                        send,
                        player_key,
                        "Invalid gold amount.".into(),
                        OutputStyle::Trade,
                    );
                    return;
                }
                trade.offer(&key, None, Some(amount));
            }
        } else {
            let item_name = args[1..].join(" ");
            let item_id = {
                let player = players.get(player_key).expect("player exists");
                find_item_in_inventory(player, &item_name)
            };
            let Some(item_id) = item_id else {
                send_output(
                    send,
                    player_key,
                    format!("Don't have \"{item_name}\"."),
                    OutputStyle::Trade,
                );
                return;
            };
            if let Some(player) = players.get_mut(player_key) {
                player.remove_item(&item_id);
            }
            trade.offer(&key, Some(&item_id), None);
        }
        send_output(
            send,
            player_key,
            "Trade offer updated.".into(),
            OutputStyle::Trade,
        );
        return;
    }

    if sub.as_deref() == Some("confirm") {
        if trade.get_session(&key).is_none() {
            send_output(send, player_key, "Not in a trade.".into(), OutputStyle::Trade);
            return;
        }
        if !trade.confirm(&key) {
            send_output(
                send,
                player_key,
                "You confirmed. Waiting for partner...".into(),
                OutputStyle::Trade,
            );
            return;
        }
        if let Some(session) = trade.get_session(&key).cloned() {
            execute_trade(&session, players, send);
            trade.cancel(&key);
        }
        return;
    }

    if !rest.is_empty() {
        let Some(target_key) = find_player_key(players, &self_username, &rest) else {
            send_output(
                send,
                player_key,
                format!("No player \"{rest}\" online."),
                OutputStyle::Trade,
            );
            return;
        };
        let target_username = players
            .get(&target_key)
            .expect("target exists")
            .username()
            .to_string();
        match trade.request(&self_username, &target_username) {
            Some(err) => send_output(send, player_key, err, OutputStyle::Trade),
            None => {
                send_output(
                    send,
                    player_key,
                    format!("Trade request sent to {target_username}."),
                    OutputStyle::Trade,
                );
                send_output(
                    send,
                    &target_key,
                    format!(
                        "{self_username} wants to trade. Type 'trade accept {self_username}'."
                    ),
                    OutputStyle::Trade,
                );
            }
        }
        return;
    }

    send_output(
        send,
        player_key,
        "Usage: trade <player> | trade accept <player> | trade offer <item|gold N> | trade confirm | trade cancel".into(),
        OutputStyle::Trade,
    );
}

fn execute_trade<F>(
    session: &TradeSession,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
) where
    F: FnMut(&str, ServerMessage),
{
    let offer_a = session.offers.get(&session.a).cloned();
    let offer_b = session.offers.get(&session.b).cloned();
    let (Some(offer_a), Some(offer_b)) = (offer_a, offer_b) else {
        return;
    };

    if !players.contains_key(&session.a) || !players.contains_key(&session.b) {
        return;
    }

    if let Some(a) = players.get_mut(&session.a) {
        a.data.gold -= offer_a.gold;
        a.data.gold += offer_b.gold;
        for item in &offer_b.items {
            a.add_item(item);
        }
    }
    if let Some(b) = players.get_mut(&session.b) {
        b.data.gold -= offer_b.gold;
        b.data.gold += offer_a.gold;
        for item in &offer_a.items {
            b.add_item(item);
        }
    }

    send_output(send, &session.a, "Trade complete!".into(), OutputStyle::Trade);
    send_output(send, &session.b, "Trade complete!".into(), OutputStyle::Trade);
}

pub fn handle_duel_command<F, R>(
    player_key: &str,
    args: &[&str],
    duel: &mut DuelManager,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
    room_notify: &mut R,
) where
    F: FnMut(&str, ServerMessage),
    R: FnMut(&str, &str, Option<&str>),
{
    let Some(player) = players.get(player_key) else {
        return;
    };
    let self_username = player.username().to_string();

    let sub = args.first().map(|s| s.to_ascii_lowercase());
    let rest = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    if sub.as_deref() == Some("accept") && !rest.is_empty() {
        let err = duel.accept(&self_username, &rest);
        if let Some(err) = err {
            send_output(send, player_key, err, OutputStyle::Combat);
        } else {
            let challenger_key = rest.to_lowercase();
            if let Some(player) = players.get_mut(player_key) {
                player.in_duel = true;
            }
            if let Some(challenger) = players.get_mut(&challenger_key) {
                challenger.in_duel = true;
            }
            send_output(
                send,
                player_key,
                format!("Duel accepted! Fight {rest}!"),
                OutputStyle::Combat,
            );
            if players.contains_key(&challenger_key) {
                send_output(
                    send,
                    &challenger_key,
                    format!("{self_username} accepted your duel!"),
                    OutputStyle::Combat,
                );
            }
            let room_id = players
                .get(player_key)
                .expect("player exists")
                .room_id()
                .to_string();
            room_notify(
                &room_id,
                &format!("{self_username} and {rest} begin a duel!"),
                None,
            );
        }
        return;
    }

    if !rest.is_empty() {
        let Some(target_key) = find_player_key(players, &self_username, &rest) else {
            send_output(
                send,
                player_key,
                format!("No player \"{rest}\" found."),
                OutputStyle::Combat,
            );
            return;
        };
        let target_username = players
            .get(&target_key)
            .expect("target exists")
            .username()
            .to_string();
        match duel.challenge(&self_username, &target_username) {
            Some(err) => send_output(send, player_key, err, OutputStyle::Combat),
            None => {
                send_output(
                    send,
                    player_key,
                    format!("You challenge {target_username} to a duel."),
                    OutputStyle::Combat,
                );
                send_output(
                    send,
                    &target_key,
                    format!(
                        "{self_username} challenges you! Type 'duel accept {self_username}'."
                    ),
                    OutputStyle::Combat,
                );
            }
        }
        return;
    }

    send_output(
        send,
        player_key,
        "Usage: duel <player> | duel accept <player>".into(),
        OutputStyle::Combat,
    );
}

pub fn handle_craft_command<F>(
    player_key: &str,
    recipe_id: &str,
    world: &World,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
) where
    F: FnMut(&str, ServerMessage),
{
    let recipe = CRAFT_RECIPES
        .iter()
        .find(|r| r.id == recipe_id || r.output == recipe_id);

    let Some(recipe) = recipe else {
        let list = CRAFT_RECIPES
            .iter()
            .map(|r| format!("  {} — {}", r.id, r.name))
            .collect::<Vec<_>>()
            .join("\n");
        send_output(
            send,
            player_key,
            format!("Recipes:\n{list}\nUsage: craft <recipe_id>"),
            OutputStyle::System,
        );
        return;
    };

    let room_id = players
        .get(player_key)
        .map(|p| p.room_id().to_string())
        .unwrap_or_default();
    let at_smith = world
        .get_room(&room_id)
        .map(|room| room.npcs.contains(&recipe.npc_id.to_string()))
        .unwrap_or(false);

    if !at_smith {
        send_output(
            send,
            player_key,
            "You must be at the smith to craft.".into(),
            OutputStyle::System,
        );
        return;
    }

    let Some(player) = players.get(player_key) else {
        return;
    };

    for (item_id, count) in &recipe.ingredients {
        if player.count_item(item_id) < *count as i32 {
            let name = ITEMS
                .get(*item_id)
                .map(|i| i.name.as_str())
                .unwrap_or(item_id);
            send_output(
                send,
                player_key,
                format!("Need {count}x {name}."),
                OutputStyle::System,
            );
            return;
        }
    }

    if player.data.gold < recipe.gold {
        send_output(
            send,
            player_key,
            format!("Need {} gold.", recipe.gold),
            OutputStyle::System,
        );
        return;
    }

    let player = players.get_mut(player_key).expect("player exists");
    for (item_id, count) in &recipe.ingredients {
        for _ in 0..*count {
            player.remove_item(item_id);
        }
    }
    player.data.gold -= recipe.gold;
    player.add_item(recipe.output);

    let output_name = ITEMS
        .get(recipe.output)
        .map(|i| i.name.as_str())
        .unwrap_or(recipe.output);
    send_output(
        send,
        player_key,
        format!("Crafted {output_name}!"),
        OutputStyle::Loot,
    );
}

pub fn handle_admin_command<F, B>(
    player_key: &str,
    args: &[&str],
    world: &World,
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
    broadcast_online: &mut B,
) where
    F: FnMut(&str, ServerMessage),
    B: FnMut(),
{
    let Some(player) = players.get(player_key) else {
        return;
    };
    if !is_admin(player.username()) {
        send_output(send, player_key, "Unknown command.".into(), OutputStyle::System);
        return;
    }

    let sub = args.first().map(|s| s.to_ascii_lowercase());
    let rest: Vec<&str> = args.get(1..).unwrap_or(&[]).to_vec();

    match sub.as_deref() {
        Some("teleport") => {
            if let Some(room_id) = rest.first() {
                if let Some(player) = players.get_mut(player_key) {
                    player.data.room_id = (*room_id).to_string();
                }
                send_output(
                    send,
                    player_key,
                    format!("Teleported to {}.", rest[0]),
                    OutputStyle::System,
                );
                broadcast_online();
            }
        }
        Some("spawn") => {
            let room_id = players
                .get(player_key)
                .map(|p| p.room_id().to_string())
                .unwrap_or_default();
            let mob_id = rest.first().copied().unwrap_or("");
            let tmpl = world.mobs().get(mob_id).cloned();
            if let Some(tmpl) = tmpl {
                world.get_room_mut(&room_id, |room| {
                    room.mobs.push(LiveMob {
                        instance_id: format!("admin_{}", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis()),
                        template_id: mob_id.to_string(),
                        hp: tmpl.hp,
                        max_hp: tmpl.hp,
                        elite: None,
                    });
                });
                send_output(
                    send,
                    player_key,
                    format!("Spawned {}.", tmpl.name),
                    OutputStyle::System,
                );
            }
        }
        Some("setlevel") => {
            if let Some(lvl_str) = rest.first() {
                if let Ok(lvl) = lvl_str.parse::<u32>() {
                    if lvl > 0 {
                        if let Some(player) = players.get_mut(player_key) {
                            player.data.level = lvl;
                        }
                        send_output(
                            send,
                            player_key,
                            format!("Level set to {lvl}."),
                            OutputStyle::System,
                        );
                    }
                }
            }
        }
        Some("reload") => match world.reload() {
            Ok(logs) => send_output(
                send,
                player_key,
                format!("World reloaded:\n{}", logs.join("\n")),
                OutputStyle::System,
            ),
            Err(e) => send_output(
                send,
                player_key,
                format!("Reload failed: {e}"),
                OutputStyle::System,
            ),
        },
        _ => {
            send_output(
                send,
                player_key,
                "Admin: teleport <room> | spawn <mob> | setlevel <n> | reload".into(),
                OutputStyle::System,
            );
        }
    }
}

pub fn handle_guild_command<F, G>(
    player_key: &str,
    args: &[&str],
    players: &mut HashMap<String, PlayerSession>,
    send: &mut F,
    global_guild_chat: &mut G,
) where
    F: FnMut(&str, ServerMessage),
    G: FnMut(&str, &str, Option<&str>),
{
    let Some(player) = players.get(player_key) else {
        return;
    };
    let self_username = player.username().to_string();

    let sub = args.first().map(|s| s.to_ascii_lowercase());
    let rest = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        String::new()
    };

    if sub.as_deref() == Some("create") && !rest.is_empty() {
        match create_guild(&self_username, &rest) {
            Err(msg) => send_output(send, player_key, msg, OutputStyle::Party),
            Ok(guild) => {
                if let Some(player) = players.get_mut(player_key) {
                    player.data.guild_id = Some(guild.id.clone());
                }
                send_output(
                    send,
                    player_key,
                    format!("Guild \"{}\" founded!", guild.name),
                    OutputStyle::Party,
                );
            }
        }
        return;
    }

    if sub.as_deref() == Some("invite") && !rest.is_empty() {
        let guild = find_guild_by_member(&self_username);
        if guild.as_ref().is_none_or(|g| !is_leader(g, &self_username)) {
            send_output(
                send,
                player_key,
                "Only guild leaders can invite.".into(),
                OutputStyle::Party,
            );
            return;
        }
        let guild = guild.expect("checked above");
        let Some(target_key) = find_player_key(players, &self_username, &rest) else {
            send_output(
                send,
                player_key,
                format!("No player \"{rest}\" found."),
                OutputStyle::Party,
            );
            return;
        };
        let target_username = players
            .get(&target_key)
            .expect("target exists")
            .username()
            .to_string();
        invite_to_guild(&guild, &target_username);
        if let Some(target) = players.get_mut(&target_key) {
            target.data.guild_id = Some(guild.id.clone());
        }
        send_output(
            send,
            player_key,
            format!("Invited {target_username} to {}.", guild.name),
            OutputStyle::Party,
        );
        send_output(
            send,
            &target_key,
            format!("You joined guild {}!", guild.name),
            OutputStyle::Party,
        );
        return;
    }

    if sub.as_deref() == Some("leave") {
        let guild = leave_guild(&self_username);
        if let Some(player) = players.get_mut(player_key) {
            player.data.guild_id = None;
        }
        let msg = guild
            .map(|g| format!("You left {}.", g.name))
            .unwrap_or_else(|| "You left the guild.".into());
        send_output(send, player_key, msg, OutputStyle::Party);
        return;
    }

    if sub.as_deref() == Some("say") && !rest.is_empty() {
        let Some(guild) = find_guild_by_member(&self_username) else {
            send_output(
                send,
                player_key,
                "You are not in a guild.".into(),
                OutputStyle::Party,
            );
            return;
        };
        global_guild_chat(
            &guild.id,
            &format!("[{}] {}: {}", guild.name, self_username, rest),
            None,
        );
        return;
    }

    let guild = find_guild_by_member(&self_username).or_else(|| {
        sub.as_ref()
            .map(|s| s.as_str())
            .and_then(find_guild_by_name)
    });
    let Some(guild) = guild else {
        send_output(
            send,
            player_key,
            "No guild. Visit the Guild Hall and: guild create <name>".into(),
            OutputStyle::Party,
        );
        return;
    };

    let members: Vec<String> = guild
        .members
        .iter()
        .map(|m| {
            players
                .get(m)
                .map(|p| p.username().to_string())
                .unwrap_or_else(|| m.clone())
        })
        .collect();

    send_output(
        send,
        player_key,
        format!(
            "[{}] Leader: {}\nMembers: {}",
            guild.name,
            guild.leader,
            members.join(", ")
        ),
        OutputStyle::Party,
    );
}

pub fn find_player_key(
    players: &HashMap<String, PlayerSession>,
    self_username: &str,
    name: &str,
) -> Option<String> {
    let lower = name.to_lowercase();
    for (key, p) in players {
        if p.authenticated
            && p.username().to_lowercase().contains(&lower)
            && !p.username().eq_ignore_ascii_case(self_username)
        {
            return Some(key.clone());
        }
    }
    None
}

pub fn find_item_in_inventory(player: &PlayerSession, name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    player.data.inventory.iter().find_map(|id| {
        let item = ITEMS.get(id);
        if id.to_lowercase().contains(&lower)
            || item
                .map(|i| i.name.to_lowercase().contains(&lower))
                .unwrap_or(false)
        {
            Some(id.clone())
        } else {
            None
        }
    })
}