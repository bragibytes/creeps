use crate::pool::{block_on, pool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub leader: String,
    pub members: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

fn row_to_guild(row: &sqlx::postgres::PgRow) -> anyhow::Result<Guild> {
    let members: Vec<String> = serde_json::from_value(row.get::<Value, _>("members"))?;
    Ok(Guild {
        id: row.get("id"),
        name: row.get("name"),
        leader: row.get("leader"),
        members,
        created_at: row.get("created_at"),
    })
}

pub fn init_guilds() -> anyhow::Result<()> {
    // Schema created by shared migrations in init_database.
    Ok(())
}

async fn fetch_guild_by_member_async(username: &str) -> anyhow::Result<Option<Guild>> {
    let key = username.to_lowercase();
    let row = sqlx::query(
        "SELECT id, name, leader, members, created_at FROM guilds WHERE members @> $1::jsonb",
    )
    .bind(serde_json::json!([key]))
    .fetch_optional(pool().as_ref())
    .await?;

    row.map(|r| row_to_guild(&r)).transpose()
}

async fn fetch_guild_by_name_async(name: &str) -> anyhow::Result<Option<Guild>> {
    let lower = name.to_lowercase();
    let row = sqlx::query(
        "SELECT id, name, leader, members, created_at FROM guilds WHERE LOWER(id) = $1 OR LOWER(name) = $1",
    )
    .bind(&lower)
    .fetch_optional(pool().as_ref())
    .await?;

    row.map(|r| row_to_guild(&r)).transpose()
}

pub fn find_guild_by_member(username: &str) -> Option<Guild> {
    block_on(fetch_guild_by_member_async(username))
        .ok()
        .flatten()
}

pub fn find_guild_by_name(name: &str) -> Option<Guild> {
    block_on(fetch_guild_by_name_async(name)).ok().flatten()
}

pub fn create_guild(leader: &str, name: &str) -> Result<Guild, String> {
    if name.len() < 3 || name.len() > 20 {
        return Err("Guild name must be 3-20 characters.".into());
    }
    if !name
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'))
    {
        return Err("Invalid guild name.".into());
    }
    if find_guild_by_member(leader).is_some() {
        return Err("You are already in a guild.".into());
    }
    if find_guild_by_name(name).is_some() {
        return Err("That guild name is taken.".into());
    }

    block_on(create_guild_async(leader, name)).map_err(|e| e.to_string())
}

async fn create_guild_async(leader: &str, name: &str) -> anyhow::Result<Guild> {
    let leader_key = leader.to_lowercase();
    let guild = Guild {
        id: name.to_lowercase().replace(' ', "_"),
        name: name.to_string(),
        leader: leader_key.clone(),
        members: vec![leader_key.clone()],
        created_at: now_iso8601(),
    };

    let members = serde_json::to_value(&guild.members)?;
    let row = sqlx::query(
        "INSERT INTO guilds (id, name, leader, members, created_at) VALUES ($1, $2, $3, $4::jsonb, $5) RETURNING id, name, leader, members, created_at",
    )
    .bind(&guild.id)
    .bind(&guild.name)
    .bind(&guild.leader)
    .bind(members)
    .bind(&guild.created_at)
    .fetch_one(pool().as_ref())
    .await?;

    row_to_guild(&row)
}

pub fn invite_to_guild(guild: &Guild, username: &str) {
    let _ = block_on(invite_to_guild_async(guild, username));
}

async fn invite_to_guild_async(guild: &Guild, username: &str) -> anyhow::Result<()> {
    let key = username.to_lowercase();
    if guild.members.iter().any(|m| m == &key) {
        return Ok(());
    }

    let mut members = guild.members.clone();
    members.push(key);
    let members_json = serde_json::to_value(&members)?;

    sqlx::query("UPDATE guilds SET members = $2::jsonb WHERE id = $1")
        .bind(&guild.id)
        .bind(members_json)
        .execute(pool().as_ref())
        .await?;

    Ok(())
}

pub fn leave_guild(username: &str) -> Option<Guild> {
    block_on(leave_guild_async(username)).ok().flatten()
}

async fn leave_guild_async(username: &str) -> anyhow::Result<Option<Guild>> {
    let key = username.to_lowercase();
    let guild = fetch_guild_by_member_async(username).await?;
    let Some(guild) = guild else {
        return Ok(None);
    };

    let mut members: Vec<String> = guild
        .members
        .iter()
        .filter(|m| *m != &key)
        .cloned()
        .collect();

    if members.is_empty() {
        sqlx::query("DELETE FROM guilds WHERE id = $1")
            .bind(&guild.id)
            .execute(pool().as_ref())
            .await?;
        return Ok(Some(guild));
    }

    let leader = if guild.leader == key {
        members[0].clone()
    } else {
        guild.leader.clone()
    };

    let members_json = serde_json::to_value(&members)?;
    sqlx::query("UPDATE guilds SET leader = $2, members = $3::jsonb WHERE id = $1")
        .bind(&guild.id)
        .bind(&leader)
        .bind(members_json)
        .execute(pool().as_ref())
        .await?;

    Ok(Some(Guild {
        leader,
        members,
        ..guild
    }))
}

pub fn get_guild_members(guild: &Guild) -> Vec<String> {
    guild.members.clone()
}

pub fn is_leader(guild: &Guild, username: &str) -> bool {
    guild.leader == username.to_lowercase()
}

pub fn find_guild(name: &str) -> Option<Guild> {
    find_guild_by_name(name)
}

pub fn list_guilds() -> Vec<Guild> {
    block_on(list_guilds_async()).unwrap_or_default()
}

async fn list_guilds_async() -> anyhow::Result<Vec<Guild>> {
    let rows = sqlx::query("SELECT id, name, leader, members, created_at FROM guilds ORDER BY name")
        .fetch_all(pool().as_ref())
        .await?;

    rows.iter().map(row_to_guild).collect()
}

pub fn guild_invite(guild: &Guild, username: &str) {
    invite_to_guild(guild, username);
}

pub fn guild_leave(username: &str) -> Option<Guild> {
    leave_guild(username)
}

pub fn guild_join(guild: &Guild, username: &str) -> Result<(), String> {
    if find_guild_by_member(username).is_some() {
        return Err("You are already in a guild.".into());
    }
    invite_to_guild(guild, username);
    Ok(())
}

pub fn disband_guild(leader: &str) -> Result<(), String> {
    let key = leader.to_lowercase();
    let guild =
        find_guild_by_member(leader).ok_or_else(|| "You are not in a guild.".to_string())?;
    if guild.leader != key {
        return Err("Only the guild leader can disband the guild.".into());
    }

    block_on(disband_guild_async(&guild.id)).map_err(|e| e.to_string())
}

async fn disband_guild_async(guild_id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM guilds WHERE id = $1")
        .bind(guild_id)
        .execute(pool().as_ref())
        .await?;
    Ok(())
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    format!("{secs}.{millis:03}Z")
}