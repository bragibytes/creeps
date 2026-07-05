use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClassName {
    Warrior,
    Mage,
    Rogue,
}

impl ClassName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warrior => "warrior",
            Self::Mage => "mage",
            Self::Rogue => "rogue",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "warrior" => Some(Self::Warrior),
            "mage" => Some(Self::Mage),
            "rogue" => Some(Self::Rogue),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStyle {
    Normal,
    System,
    Combat,
    Chat,
    Quest,
    Loot,
    Death,
    Party,
    Trade,
    Global,
    Emote,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        password: String,
        #[serde(rename = "className")]
        class_name: ClassName,
    },
    Command {
        input: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub username: String,
    #[serde(rename = "className")]
    pub class_name: ClassName,
    pub level: u32,
    pub hp: i32,
    #[serde(rename = "maxHp")]
    pub max_hp: i32,
    pub mp: i32,
    #[serde(rename = "maxMp")]
    pub max_mp: i32,
    pub xp: i32,
    #[serde(rename = "xpToLevel")]
    pub xp_to_level: i32,
    pub gold: i32,
    pub room: String,
    #[serde(rename = "roomName", skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(rename = "inCombat", skip_serializing_if = "Option::is_none")]
    pub in_combat: Option<bool>,
    #[serde(rename = "partyLeader", skip_serializing_if = "Option::is_none")]
    pub party_leader: Option<String>,
    #[serde(rename = "inDuel", skip_serializing_if = "Option::is_none")]
    pub in_duel: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "guildName", skip_serializing_if = "Option::is_none")]
    pub guild_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlinePlayer {
    pub username: String,
    pub level: u32,
    #[serde(rename = "className")]
    pub class_name: ClassName,
    pub zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapCell {
    pub id: String,
    #[serde(rename = "mapX")]
    pub map_x: i32,
    #[serde(rename = "mapY")]
    pub map_y: i32,
    pub name: String,
    pub current: bool,
    #[serde(rename = "hasExit")]
    pub has_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSnapshot {
    #[serde(rename = "inCombat")]
    pub in_combat: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(rename = "targetHp", skip_serializing_if = "Option::is_none")]
    pub target_hp: Option<i32>,
    #[serde(rename = "targetMaxHp", skip_serializing_if = "Option::is_none")]
    pub target_max_hp: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    Banner,
    Prompt {
        text: String,
    },
    Output {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<OutputStyle>,
    },
    Room {
        title: String,
        description: String,
        exits: String,
        entities: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        zone: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        minimap: Option<Vec<MinimapCell>>,
        #[serde(rename = "zoneArt", skip_serializing_if = "Option::is_none")]
        zone_art: Option<String>,
    },
    Stats {
        player: PlayerSnapshot,
    },
    Online {
        players: Vec<OnlinePlayer>,
    },
    Flash {
        color: String,
    },
    Bell,
    Ticker {
        text: String,
    },
    Motd {
        text: String,
    },
    Combat {
        state: CombatSnapshot,
    },
    Error {
        text: String,
    },
    Disconnect {
        reason: String,
    },
}