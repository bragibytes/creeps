use std::collections::{HashMap, HashSet};

use crate::player::PlayerSession;

pub struct PartyManager {
    leader_of: HashMap<String, HashSet<String>>,
    member_of: HashMap<String, String>,
    invites: HashMap<String, String>,
}

impl Default for PartyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PartyManager {
    pub fn new() -> Self {
        Self {
            leader_of: HashMap::new(),
            member_of: HashMap::new(),
            invites: HashMap::new(),
        }
    }

    pub fn invite(&mut self, leader: &PlayerSession, target: &PlayerSession) -> Option<String> {
        let target_key = target.username().to_lowercase();
        if self.member_of.contains_key(&target_key) {
            return Some(format!("{} is already in a party.", target.username()));
        }

        let leader_key = leader.username().to_lowercase();
        if let Some(member_leader) = self.member_of.get(&leader_key) {
            if member_leader != &leader_key {
                return Some("Only the party leader can invite.".into());
            }
        }

        self.ensure_party(leader.username());
        self.invites.insert(target_key, leader_key);
        None
    }

    pub fn join(&mut self, player: &PlayerSession, leader_name: &str) -> Option<String> {
        let leader_key = leader_name.to_lowercase();
        let player_key = player.username().to_lowercase();

        if self.invites.get(&player_key) != Some(&leader_key) {
            return Some(format!("No party invite from {leader_name}."));
        }

        self.invites.remove(&player_key);
        self.ensure_party(leader_name);
        self.leader_of
            .get_mut(&leader_key)
            .expect("party exists after ensure_party")
            .insert(player_key.clone());
        self.member_of.insert(player_key, leader_key);
        None
    }

    pub fn leave(&mut self, player: &PlayerSession) {
        let player_key = player.username().to_lowercase();
        let Some(leader_key) = self.member_of.get(&player_key).cloned() else {
            return;
        };

        if let Some(party) = self.leader_of.get_mut(&leader_key) {
            party.remove(&player_key);
        }
        self.member_of.remove(&player_key);

        if leader_key == player_key {
            let members: Vec<String> = self
                .leader_of
                .get(&leader_key)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();

            if let Some(new_leader) = members.first().cloned() {
                let rest: HashSet<String> = members.iter().skip(1).cloned().collect();
                self.leader_of.remove(&leader_key);
                self.leader_of.insert(new_leader.clone(), rest);
                for member in &members {
                    self.member_of
                        .insert(member.clone(), new_leader.clone());
                }
            } else {
                self.leader_of.remove(&leader_key);
            }
        }
    }

    pub fn get_leader_key(&self, player: &PlayerSession) -> Option<String> {
        self.member_of.get(&player.username().to_lowercase()).cloned()
    }

    pub fn get_member_usernames(
        &self,
        leader_key: &str,
        players: &HashMap<String, PlayerSession>,
    ) -> Vec<String> {
        let leader_key = leader_key.to_lowercase();
        let Some(party) = self.leader_of.get(&leader_key) else {
            return players
                .get(&leader_key)
                .map(|p| vec![p.username().to_string()])
                .unwrap_or_default();
        };

        let Some(leader) = players.get(&leader_key) else {
            return Vec::new();
        };

        let mut names = vec![leader.username().to_string()];
        for member_key in party {
            if let Some(p) = players.get(member_key) {
                names.push(p.username().to_string());
            }
        }
        names
    }

    pub fn get_party_peers(
        &self,
        player: &PlayerSession,
        players: &HashMap<String, PlayerSession>,
    ) -> Vec<PlayerSession> {
        let leader_key = self
            .member_of
            .get(&player.username().to_lowercase())
            .cloned()
            .unwrap_or_else(|| player.username().to_lowercase());

        self.get_member_usernames(&leader_key, players)
            .into_iter()
            .filter_map(|name| players.get(&name.to_lowercase()).cloned())
            .filter(|p| p.authenticated)
            .collect()
    }

    fn ensure_party(&mut self, leader_username: &str) {
        let key = leader_username.to_lowercase();
        if !self.leader_of.contains_key(&key) {
            self.leader_of.insert(key.clone(), HashSet::new());
            self.member_of.insert(key.clone(), key);
        }
    }

    pub fn on_disconnect(&mut self, player: &PlayerSession) {
        let player_key = player.username().to_lowercase();
        self.leave(player);
        self.invites.remove(&player_key);

        let stale: Vec<String> = self
            .invites
            .iter()
            .filter_map(|(target, leader)| {
                if leader == &player_key {
                    Some(target.clone())
                } else {
                    None
                }
            })
            .collect();

        for target in stale {
            self.invites.remove(&target);
        }
    }
}