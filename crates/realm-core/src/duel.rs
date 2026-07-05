use std::collections::HashMap;

pub struct DuelManager {
    challenges: HashMap<String, String>,
    active: HashMap<String, String>,
}

impl Default for DuelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DuelManager {
    pub fn new() -> Self {
        Self {
            challenges: HashMap::new(),
            active: HashMap::new(),
        }
    }

    pub fn challenge(&mut self, challenger: &str, target: &str) -> Option<String> {
        if challenger.eq_ignore_ascii_case(target) {
            return Some("You cannot duel yourself.".into());
        }

        self.challenges
            .insert(target.to_lowercase(), challenger.to_lowercase());
        None
    }

    pub fn accept(&mut self, accepter: &str, challenger: &str) -> Option<String> {
        let accepter_key = accepter.to_lowercase();
        let challenger_key = challenger.to_lowercase();

        if self.challenges.get(&accepter_key) != Some(&challenger_key) {
            return Some(format!("No duel challenge from {challenger}."));
        }

        self.challenges.remove(&accepter_key);
        self.active.insert(accepter_key.clone(), challenger_key.clone());
        self.active.insert(challenger_key, accepter_key);
        None
    }

    pub fn is_in_duel(&self, username: &str) -> bool {
        self.active.contains_key(&username.to_lowercase())
    }

    pub fn get_opponent(&self, username: &str) -> Option<String> {
        self.active.get(&username.to_lowercase()).cloned()
    }

    pub fn end_duel(&mut self, a: &str, b: &str) {
        self.active.remove(&a.to_lowercase());
        self.active.remove(&b.to_lowercase());
    }

    pub fn on_disconnect(&mut self, username: &str) {
        let username_key = username.to_lowercase();

        if let Some(opponent) = self.active.get(&username_key).cloned() {
            self.active.remove(&username_key);
            self.active.remove(&opponent);
        }

        self.challenges.remove(&username_key);

        let stale: Vec<String> = self
            .challenges
            .iter()
            .filter_map(|(target, challenger)| {
                if challenger == &username_key {
                    Some(target.clone())
                } else {
                    None
                }
            })
            .collect();

        for target in stale {
            self.challenges.remove(&target);
        }
    }
}