use realm_protocol::ClassName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::East => "east",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "north" | "n" => Some(Self::North),
            "south" | "s" => Some(Self::South),
            "east" | "e" => Some(Self::East),
            "west" | "w" => Some(Self::West),
            "up" | "u" => Some(Self::Up),
            "down" | "d" => Some(Self::Down),
            _ => None,
        }
    }
}

pub static DIRECTION_ALIASES: LazyLock<HashMap<String, Direction>> = LazyLock::new(|| {
    HashMap::from([
        ("n".into(), Direction::North),
        ("north".into(), Direction::North),
        ("s".into(), Direction::South),
        ("south".into(), Direction::South),
        ("e".into(), Direction::East),
        ("east".into(), Direction::East),
        ("w".into(), Direction::West),
        ("west".into(), Direction::West),
        ("u".into(), Direction::Up),
        ("up".into(), Direction::Up),
        ("d".into(), Direction::Down),
        ("down".into(), Direction::Down),
    ])
});

pub static HOTKEY_COMMANDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from([
        ("n".into(), "north".into()),
        ("s".into(), "south".into()),
        ("e".into(), "east".into()),
        ("w".into(), "west".into()),
        ("u".into(), "up".into()),
        ("d".into(), "down".into()),
        ("l".into(), "look".into()),
        ("i".into(), "inventory".into()),
        ("h".into(), "help".into()),
        ("g".into(), "global".into()),
    ])
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LootRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heal: Option<i32>,
    pub value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<LootRarity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldRange {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDrop {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub chance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: u32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub xp: i32,
    pub gold: GoldRange,
    #[serde(default)]
    pub loot: Vec<LootDrop>,
    pub hostile: bool,
    #[serde(rename = "respawnSeconds")]
    pub respawn_seconds: u64,
    #[serde(default)]
    pub elite: bool,
    #[serde(default)]
    pub boss: bool,
}

pub static RARITY_COLORS: LazyLock<HashMap<LootRarity, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (LootRarity::Common, "white"),
        (LootRarity::Uncommon, "green"),
        (LootRarity::Rare, "blue"),
        (LootRarity::Epic, "magenta"),
    ])
});

#[derive(Debug, Clone)]
pub struct LockedExit {
    pub item: &'static str,
    pub message: &'static str,
}

pub static LOCKED_EXITS: LazyLock<HashMap<&'static str, LockedExit>> = LazyLock::new(|| {
    HashMap::from([(
        "crypt_stairs",
        LockedExit {
            item: "ancient_key",
            message: "The iron door is locked. You need the Ancient Key.",
        },
    )])
});

pub static ZONE_ART: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("Eldermoor", "  🏰⚔️🏰  "),
        ("Whispering Woods", "  🌲🌳🌲  "),
        ("Ironspine Mountains", "  ⛰️🦅⛰️  "),
        ("Crypt of Ash", "  💀⚰️💀  "),
    ])
});

#[derive(Debug, Clone)]
pub struct CraftRecipe {
    pub id: &'static str,
    pub name: &'static str,
    pub output: &'static str,
    pub ingredients: HashMap<&'static str, u32>,
    pub gold: i32,
    pub npc_id: &'static str,
}

pub static CRAFT_RECIPES: LazyLock<Vec<CraftRecipe>> = LazyLock::new(|| {
    vec![
        CraftRecipe {
            id: "craft_leather",
            name: "Leather Armor",
            output: "leather_armor",
            ingredients: HashMap::from([("wolf_pelt", 3)]),
            gold: 10,
            npc_id: "eldermoor_smith",
        },
        CraftRecipe {
            id: "craft_iron_sword",
            name: "Iron Sword",
            output: "iron_sword",
            ingredients: HashMap::from([("goblin_ear", 5)]),
            gold: 25,
            npc_id: "eldermoor_smith",
        },
    ]
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopEntry {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub price: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub greeting: String,
    #[serde(rename = "questId", skip_serializing_if = "Option::is_none")]
    pub quest_id: Option<String>,
    #[serde(default)]
    pub shop: Vec<ShopEntry>,
    #[serde(default)]
    pub crafts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub zone: String,
    #[serde(rename = "mapX", skip_serializing_if = "Option::is_none")]
    pub map_x: Option<i32>,
    #[serde(rename = "mapY", skip_serializing_if = "Option::is_none")]
    pub map_y: Option<i32>,
    pub exits: HashMap<String, String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub mobs: Vec<String>,
    #[serde(default)]
    pub npcs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    #[serde(rename = "type")]
    pub objective_type: String,
    pub target: String,
    pub count: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRewards {
    pub xp: i32,
    pub gold: i32,
    #[serde(default)]
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "giverNpc")]
    pub giver_npc: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
}

#[derive(Debug, Clone)]
pub struct ClassStats {
    pub name: ClassName,
    pub display_name: &'static str,
    pub max_hp: i32,
    pub max_mp: i32,
    pub attack: i32,
    pub defense: i32,
    pub ability: &'static str,
    pub ability_damage: i32,
    pub ability_cost: i32,
    pub ability2: &'static str,
    pub ability2_damage: i32,
    pub ability2_cost: i32,
    pub ability2_level: u32,
    pub description: &'static str,
    pub art: &'static [&'static str],
}

pub static CLASSES: LazyLock<HashMap<ClassName, ClassStats>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        ClassName::Warrior,
        ClassStats {
            name: ClassName::Warrior,
            display_name: "Warrior",
            max_hp: 120,
            max_mp: 20,
            attack: 12,
            defense: 8,
            ability: "Power Strike",
            ability_damage: 25,
            ability_cost: 10,
            ability2: "Shield Bash",
            ability2_damage: 20,
            ability2_cost: 15,
            ability2_level: 5,
            description: "A stalwart fighter with high HP and crushing melee attacks.",
            art: &["  ⚔️🛡️  ", " /|==|\\ ", "  / \\  "],
        },
    );
    m.insert(
        ClassName::Mage,
        ClassStats {
            name: ClassName::Mage,
            display_name: "Mage",
            max_hp: 70,
            max_mp: 80,
            attack: 8,
            defense: 3,
            ability: "Fireball",
            ability_damage: 35,
            ability_cost: 15,
            ability2: "Chain Lightning",
            ability2_damage: 45,
            ability2_cost: 25,
            ability2_level: 5,
            description: "A wielder of arcane fire, devastating from range but fragile.",
            art: &["  🔮✨  ", "  /|\\  ", "  / \\  "],
        },
    );
    m.insert(
        ClassName::Rogue,
        ClassStats {
            name: ClassName::Rogue,
            display_name: "Rogue",
            max_hp: 90,
            max_mp: 40,
            attack: 14,
            defense: 5,
            ability: "Backstab",
            ability_damage: 30,
            ability_cost: 12,
            ability2: "Smoke Bomb",
            ability2_damage: 25,
            ability2_cost: 18,
            ability2_level: 5,
            description: "A swift shadow who strikes from the darkness with deadly precision.",
            art: &["  🗡️💨  ", "  /|\\  ", "  / \\  "],
        },
    );
    m
});

pub fn class_stats(class_name: ClassName) -> &'static ClassStats {
    &CLASSES[&class_name]
}

pub fn xp_for_level(level: u32) -> i32 {
    (100.0 * 1.5_f64.powi(level as i32 - 1)).floor() as i32
}