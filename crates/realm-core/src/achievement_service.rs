use realm_protocol::{OutputStyle, ServerMessage};

use crate::achievements::{grant_achievement, AchievementDef};
use crate::player::PlayerSession;

#[derive(Debug, Clone, Default)]
pub struct AchievementTriggers {
    pub kill: Option<String>,
    pub room: Option<String>,
    pub gold: Option<i32>,
    pub level: Option<u32>,
    pub duel_win: Option<bool>,
}

pub fn check_achievements<F>(player: &mut PlayerSession, mut send: F, triggers: AchievementTriggers)
where
    F: FnMut(&str, ServerMessage),
{
    if triggers.kill.is_some() && player.data.kills == 1 {
        let achievement = grant_achievement(&mut player.data.achievements, "first_blood");
        notify(player, &mut send, achievement);
    }

    if triggers
        .kill
        .as_ref()
        .is_some_and(|k| k.contains("goblin"))
        && player.data.goblin_kills >= 10
    {
        let achievement = grant_achievement(&mut player.data.achievements, "goblin_hunter");
        notify(player, &mut send, achievement);
    }

    if triggers.kill.as_deref() == Some("goblin_chief") {
        let achievement = grant_achievement(&mut player.data.achievements, "chief_slayer");
        notify(player, &mut send, achievement);
    }

    if triggers.room.as_deref() == Some("crystal_cavern") {
        let achievement = grant_achievement(&mut player.data.achievements, "explorer");
        notify(player, &mut send, achievement);
    }

    if triggers.gold.is_some_and(|g| g >= 500) {
        let achievement = grant_achievement(&mut player.data.achievements, "wealthy");
        notify(player, &mut send, achievement);
    }

    if triggers.level.is_some_and(|l| l >= 10) {
        let achievement = grant_achievement(&mut player.data.achievements, "veteran");
        notify(player, &mut send, achievement);
    }

    if triggers.duel_win == Some(true) {
        let achievement = grant_achievement(&mut player.data.achievements, "duelist");
        notify(player, &mut send, achievement);
    }
}

fn notify<F>(player: &mut PlayerSession, send: &mut F, achievement: Option<AchievementDef>)
where
    F: FnMut(&str, ServerMessage),
{
    let Some(achievement) = achievement else {
        return;
    };

    let username = player.username().to_string();

    send(
        &username,
        ServerMessage::Output {
            text: format!(
                "*** ACHIEVEMENT: {} ***\n{}",
                achievement.name, achievement.description
            ),
            style: Some(OutputStyle::Quest),
        },
    );

    if let Some(title) = achievement.title {
        player.data.title = Some(title.to_string());
        send(
            &username,
            ServerMessage::Output {
                text: format!("Title earned: {title}"),
                style: Some(OutputStyle::Loot),
            },
        );
    }

    send(&username, ServerMessage::Bell);
}

pub fn format_achievements(player: &PlayerSession) -> String {
    if player.data.achievements.is_empty() {
        return "No achievements yet. Explore, fight, and conquer!".into();
    }

    let mut lines = vec!["-- Achievements --".into()];

    for id in &player.data.achievements {
        if let Some(a) = crate::achievements::achievements().get(id.as_str()) {
            lines.push(format!("  ✓ {} — {}", a.name, a.description));
        }
    }

    if let Some(title) = &player.data.title {
        lines.push(format!("\nTitle: {title}"));
    }

    lines.join("\n")
}