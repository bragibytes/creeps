use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub title: Option<&'static str>,
}

pub fn achievements() -> &'static HashMap<&'static str, AchievementDef> {
    static ACHIEVEMENTS: OnceLock<HashMap<&'static str, AchievementDef>> = OnceLock::new();
    ACHIEVEMENTS.get_or_init(|| {
        HashMap::from([
            (
                "first_blood",
                AchievementDef {
                    id: "first_blood",
                    name: "First Blood",
                    description: "Slay your first creature.",
                    title: Some("Initiate"),
                },
            ),
            (
                "goblin_hunter",
                AchievementDef {
                    id: "goblin_hunter",
                    name: "Goblin Hunter",
                    description: "Slay 10 goblins.",
                    title: Some("Goblin Hunter"),
                },
            ),
            (
                "chief_slayer",
                AchievementDef {
                    id: "chief_slayer",
                    name: "Kingslayer",
                    description: "Defeat Goblin Chief Grak.",
                    title: Some("Giantkiller"),
                },
            ),
            (
                "duelist",
                AchievementDef {
                    id: "duelist",
                    name: "Duelist",
                    description: "Win your first duel.",
                    title: Some("Duelist"),
                },
            ),
            (
                "explorer",
                AchievementDef {
                    id: "explorer",
                    name: "Explorer",
                    description: "Visit the Crystal Cavern.",
                    title: Some("Pathfinder"),
                },
            ),
            (
                "wealthy",
                AchievementDef {
                    id: "wealthy",
                    name: "Wealthy",
                    description: "Accumulate 500 gold.",
                    title: Some("Magnate"),
                },
            ),
            (
                "veteran",
                AchievementDef {
                    id: "veteran",
                    name: "Veteran",
                    description: "Reach level 10.",
                    title: Some("Veteran"),
                },
            ),
        ])
    })
}

pub fn grant_achievement(earned: &mut Vec<String>, id: &str) -> Option<AchievementDef> {
    if earned.iter().any(|e| e == id) {
        return None;
    }

    earned.push(id.to_string());
    achievements().get(id).cloned()
}