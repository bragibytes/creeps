use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TradeOffer {
    pub items: Vec<String>,
    pub gold: i32,
}

#[derive(Debug, Clone)]
pub struct TradeSession {
    pub a: String,
    pub b: String,
    pub offers: HashMap<String, TradeOffer>,
    pub confirmed: HashSet<String>,
}

pub struct TradeManager {
    sessions: HashMap<String, TradeSession>,
    player_trade: HashMap<String, String>,
    pending: HashMap<String, String>,
}

impl Default for TradeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            player_trade: HashMap::new(),
            pending: HashMap::new(),
        }
    }

    fn key(&self, a: &str, b: &str) -> String {
        let mut parts = [a.to_lowercase(), b.to_lowercase()];
        parts.sort();
        format!("{}:{}", parts[0], parts[1])
    }

    pub fn request(&mut self, from: &str, to: &str) -> Option<String> {
        let from_key = from.to_lowercase();
        let to_key = to.to_lowercase();

        if from_key == to_key {
            return Some("You cannot trade with yourself.".into());
        }
        if self.player_trade.contains_key(&from_key) {
            return Some("You are already in a trade.".into());
        }
        if self.player_trade.contains_key(&to_key) {
            return Some("That player is busy trading.".into());
        }

        self.pending.insert(to_key, from_key);
        None
    }

    pub fn accept(&mut self, accepter: &str, requester: &str) -> Result<TradeSession, String> {
        let accepter_key = accepter.to_lowercase();
        let requester_key = requester.to_lowercase();

        if self.pending.get(&accepter_key) != Some(&requester_key) {
            return Err(format!("No trade request from {requester}."));
        }

        self.pending.remove(&accepter_key);
        let session_key = self.key(accepter, requester);

        let session = TradeSession {
            a: requester_key.clone(),
            b: accepter_key.clone(),
            offers: HashMap::from([
                (
                    requester_key.clone(),
                    TradeOffer {
                        items: Vec::new(),
                        gold: 0,
                    },
                ),
                (
                    accepter_key.clone(),
                    TradeOffer {
                        items: Vec::new(),
                        gold: 0,
                    },
                ),
            ]),
            confirmed: HashSet::new(),
        };

        self.sessions.insert(session_key.clone(), session.clone());
        self.player_trade.insert(requester_key, session_key.clone());
        self.player_trade.insert(accepter_key, session_key);
        Ok(session)
    }

    pub fn get_session(&self, username: &str) -> Option<&TradeSession> {
        let key = self.player_trade.get(&username.to_lowercase())?;
        self.sessions.get(key)
    }

    pub fn get_partner(&self, session: &TradeSession, username: &str) -> String {
        let key = username.to_lowercase();
        if session.a == key {
            session.b.clone()
        } else {
            session.a.clone()
        }
    }

    pub fn offer(
        &mut self,
        username: &str,
        item_id: Option<&str>,
        gold: Option<i32>,
    ) -> Option<String> {
        let username_key = username.to_lowercase();
        let session_key = self.player_trade.get(&username_key)?.clone();
        let session = self.sessions.get_mut(&session_key)?;

        let offer = session.offers.get_mut(&username_key)?;
        if let Some(item) = item_id {
            offer.items.push(item.to_string());
        }
        if let Some(g) = gold {
            if g > 0 {
                offer.gold = g;
            }
        }
        session.confirmed.clear();
        None
    }

    pub fn confirm(&mut self, username: &str) -> bool {
        let username_key = username.to_lowercase();
        let Some(session_key) = self.player_trade.get(&username_key).cloned() else {
            return false;
        };
        let Some(session) = self.sessions.get_mut(&session_key) else {
            return false;
        };

        session.confirmed.insert(username_key);
        session.confirmed.contains(&session.a) && session.confirmed.contains(&session.b)
    }

    pub fn cancel(&mut self, username: &str) {
        let username_key = username.to_lowercase();

        if let Some(key) = self.player_trade.get(&username_key).cloned() {
            if let Some(session) = self.sessions.get(&key) {
                self.player_trade.remove(&session.a);
                self.player_trade.remove(&session.b);
                self.sessions.remove(&key);
            }
        }

        let stale: Vec<String> = self
            .pending
            .iter()
            .filter_map(|(to, from)| {
                if from == &username_key || to == &username_key {
                    Some(to.clone())
                } else {
                    None
                }
            })
            .collect();

        for to in stale {
            self.pending.remove(&to);
        }
    }

    pub fn on_disconnect(&mut self, username: &str) {
        self.cancel(username);
    }
}