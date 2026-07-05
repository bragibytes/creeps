use crate::types::QuestTemplate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestStatus {
    Active,
    Complete,
    #[serde(rename = "turned_in")]
    TurnedIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    #[serde(rename = "questId")]
    pub quest_id: String,
    pub status: QuestStatus,
    pub progress: HashMap<String, i32>,
}

pub fn create_quest_progress(quest: &QuestTemplate) -> QuestProgress {
    let progress = quest
        .objectives
        .iter()
        .map(|obj| (obj.target.clone(), 0))
        .collect();
    QuestProgress {
        quest_id: quest.id.clone(),
        status: QuestStatus::Active,
        progress,
    }
}

pub fn check_quest_complete(quest: &QuestTemplate, qp: &QuestProgress) -> bool {
    quest.objectives.iter().all(|obj| {
        qp.progress.get(&obj.target).copied().unwrap_or(0) >= obj.count
    })
}

pub fn format_quest_progress(quest: &QuestTemplate, qp: &QuestProgress) -> String {
    let mut lines = vec![
        format!("[{}] {}", quest.name, quest.description),
        "Objectives:".to_string(),
    ];
    for obj in &quest.objectives {
        let current = qp.progress.get(&obj.target).copied().unwrap_or(0);
        let done = current >= obj.count;
        let marker = if done { "✓" } else { "○" };
        lines.push(format!(
            "  {marker} {} ({current}/{})",
            obj.description, obj.count
        ));
    }
    lines.join("\n")
}