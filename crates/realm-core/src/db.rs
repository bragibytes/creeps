use crate::pool::{block_on, connect_and_migrate, pool};
use crate::quests::QuestProgress;
use realm_protocol::ClassName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor: Option<String>,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            weapon: None,
            armor: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPlayer {
    pub id: u32,
    pub username: String,
    #[serde(rename = "passwordHash")]
    pub password_hash: String,
    #[serde(rename = "className")]
    pub class_name: ClassName,
    pub level: u32,
    pub xp: i32,
    pub hp: i32,
    #[serde(rename = "maxHp")]
    pub max_hp: i32,
    pub mp: i32,
    #[serde(rename = "maxMp")]
    pub max_mp: i32,
    pub gold: i32,
    #[serde(rename = "roomId")]
    pub room_id: String,
    pub inventory: Vec<String>,
    pub equipment: Equipment,
    pub quests: Vec<QuestProgress>,
    #[serde(default)]
    pub achievements: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub kills: u32,
    #[serde(default)]
    pub deaths: u32,
    #[serde(rename = "guildId", default)]
    pub guild_id: Option<String>,
    #[serde(rename = "goblinKills", default)]
    pub goblin_kills: u32,
}

fn normalize_player(mut player: StoredPlayer) -> StoredPlayer {
    if player.achievements.is_empty() {
        player.achievements = Vec::new();
    }
    player
}

fn row_to_player(row: &sqlx::postgres::PgRow) -> anyhow::Result<StoredPlayer> {
    let class_name = match row.get::<String, _>("class_name").to_lowercase().as_str() {
        "warrior" => ClassName::Warrior,
        "mage" => ClassName::Mage,
        "rogue" => ClassName::Rogue,
        other => anyhow::bail!("unknown class_name: {other}"),
    };

    let inventory: Vec<String> = serde_json::from_value(row.get::<Value, _>("inventory"))?;
    let equipment: Equipment = serde_json::from_value(row.get::<Value, _>("equipment"))?;
    let quests: Vec<QuestProgress> = serde_json::from_value(row.get::<Value, _>("quests"))?;
    let achievements: Vec<String> = serde_json::from_value(row.get::<Value, _>("achievements"))?;

    Ok(normalize_player(StoredPlayer {
        id: row.get::<i32, _>("id") as u32,
        username: row.get("username"),
        password_hash: row.get("password_hash"),
        class_name,
        level: row.get::<i32, _>("level") as u32,
        xp: row.get("xp"),
        hp: row.get("hp"),
        max_hp: row.get("max_hp"),
        mp: row.get("mp"),
        max_mp: row.get("max_mp"),
        gold: row.get("gold"),
        room_id: row.get("room_id"),
        inventory,
        equipment,
        quests,
        achievements,
        title: row.get("title"),
        kills: row.get::<i32, _>("kills") as u32,
        deaths: row.get::<i32, _>("deaths") as u32,
        guild_id: row.get("guild_id"),
        goblin_kills: row.get::<i32, _>("goblin_kills") as u32,
    }))
}

pub async fn init_database() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL is required (set in .env or Railway Postgres)"))?;
    connect_and_migrate(&database_url).await?;
    Ok(())
}

pub fn find_player(username: &str) -> Option<StoredPlayer> {
    block_on(find_player_async(username)).ok().flatten()
}

async fn find_player_async(username: &str) -> anyhow::Result<Option<StoredPlayer>> {
    let key = username.to_lowercase();
    let row = sqlx::query(
        "SELECT id, username, password_hash, class_name, level, xp, hp, max_hp, mp, max_mp, gold, room_id, inventory, equipment, quests, achievements, title, kills, deaths, guild_id, goblin_kills FROM players WHERE LOWER(username) = $1",
    )
    .bind(&key)
    .fetch_optional(pool().as_ref())
    .await?;

    row.map(|r| row_to_player(&r)).transpose()
}

pub fn get_all_players() -> Vec<StoredPlayer> {
    block_on(get_all_players_async()).unwrap_or_default()
}

async fn get_all_players_async() -> anyhow::Result<Vec<StoredPlayer>> {
    let rows = sqlx::query(
        "SELECT id, username, password_hash, class_name, level, xp, hp, max_hp, mp, max_mp, gold, room_id, inventory, equipment, quests, achievements, title, kills, deaths, guild_id, goblin_kills FROM players ORDER BY id",
    )
    .fetch_all(pool().as_ref())
    .await?;

    rows.iter().map(row_to_player).collect()
}

pub fn create_player(
    username: String,
    password_hash: String,
    class_name: ClassName,
    stats: (i32, i32),
) -> StoredPlayer {
    block_on(create_player_async(
        username,
        password_hash,
        class_name,
        stats,
    ))
    .expect("create_player failed")
}

async fn create_player_async(
    username: String,
    password_hash: String,
    class_name: ClassName,
    stats: (i32, i32),
) -> anyhow::Result<StoredPlayer> {
    let inventory = serde_json::json!([]);
    let equipment = serde_json::to_value(Equipment::default())?;
    let quests = serde_json::json!([]);
    let achievements = serde_json::json!([]);

    let row = sqlx::query(
        "INSERT INTO players (username, password_hash, class_name, hp, max_hp, mp, max_mp, inventory, equipment, quests, achievements) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, $9::jsonb, $10::jsonb, $11::jsonb) RETURNING id, username, password_hash, class_name, level, xp, hp, max_hp, mp, max_mp, gold, room_id, inventory, equipment, quests, achievements, title, kills, deaths, guild_id, goblin_kills",
    )
    .bind(&username)
    .bind(&password_hash)
    .bind(class_name.as_str())
    .bind(stats.0)
    .bind(stats.0)
    .bind(stats.1)
    .bind(stats.1)
    .bind(inventory)
    .bind(equipment)
    .bind(quests)
    .bind(achievements)
    .fetch_one(pool().as_ref())
    .await?;

    row_to_player(&row)
}

pub fn save_player(player: &StoredPlayer) -> anyhow::Result<()> {
    block_on(save_player_async(player))
}

async fn save_player_async(player: &StoredPlayer) -> anyhow::Result<()> {
    let inventory = serde_json::to_value(&player.inventory)?;
    let equipment = serde_json::to_value(&player.equipment)?;
    let quests = serde_json::to_value(&player.quests)?;
    let achievements = serde_json::to_value(&player.achievements)?;

    sqlx::query(
        "UPDATE players SET username = $2, password_hash = $3, class_name = $4, level = $5, xp = $6, hp = $7, max_hp = $8, mp = $9, max_mp = $10, gold = $11, room_id = $12, inventory = $13::jsonb, equipment = $14::jsonb, quests = $15::jsonb, achievements = $16::jsonb, title = $17, kills = $18, deaths = $19, guild_id = $20, goblin_kills = $21, updated_at = NOW() WHERE id = $1",
    )
    .bind(player.id as i32)
    .bind(&player.username)
    .bind(&player.password_hash)
    .bind(player.class_name.as_str())
    .bind(player.level as i32)
    .bind(player.xp)
    .bind(player.hp)
    .bind(player.max_hp)
    .bind(player.mp)
    .bind(player.max_mp)
    .bind(player.gold)
    .bind(&player.room_id)
    .bind(inventory)
    .bind(equipment)
    .bind(quests)
    .bind(achievements)
    .bind(&player.title)
    .bind(player.kills as i32)
    .bind(player.deaths as i32)
    .bind(&player.guild_id)
    .bind(player.goblin_kills as i32)
    .execute(pool().as_ref())
    .await?;

    Ok(())
}

pub fn hash_password(password: &str) -> String {
    let mut hash: i32 = 0;
    for ch in password.chars() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(ch as u32 as i32);
    }
    format!("h{}", js_to_string_radix(hash, 36))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

fn js_to_string_radix(mut n: i32, radix: u32) -> String {
    if radix < 2 || radix > 36 {
        return n.to_string();
    }
    if n == 0 {
        return "0".into();
    }
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let negative = n < 0;
    let mut digits = Vec::new();
    while n != 0 {
        let rem = (n % radix as i32).unsigned_abs() as usize;
        digits.push(DIGITS[rem]);
        n /= radix as i32;
    }
    if negative {
        digits.push(b'-');
    }
    digits.reverse();
    String::from_utf8(digits).unwrap_or_else(|_| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_is_stable() {
        assert_eq!(hash_password("test"), hash_password("test"));
    }
}