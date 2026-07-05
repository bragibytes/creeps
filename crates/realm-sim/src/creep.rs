use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BodyPart {
    Move,
    Work,
    Carry,
    Attack,
    RangedAttack,
    Heal,
    Claim,
    Tough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodySegment {
    pub part: BodyPart,
    pub hits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CreepAction {
    Idle,
    Move { x: i32, y: i32 },
    Harvest { structure_id: String },
    Transfer { structure_id: String },
    Build { structure_id: String },
    Repair { structure_id: String },
    Attack { creep_id: String },
    ClaimController,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creep {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub room: String,
    pub pos: Position,
    pub body: Vec<BodySegment>,
    pub fatigue: u32,
    pub action: CreepAction,
    #[serde(rename = "carryingEnergy")]
    pub carrying_energy: i32,
    #[serde(rename = "carryingCapacity")]
    pub carrying_capacity: i32,
}

impl Creep {
    pub fn carrying_capacity(body: &[BodySegment]) -> i32 {
        body.iter()
            .filter(|s| s.part == BodyPart::Carry && s.hits > 0)
            .count() as i32
            * 50
    }

    pub fn move_cost(body: &[BodySegment]) -> u32 {
        let total: u32 = body.iter().map(|s| s.hits).sum();
        let moves = body
            .iter()
            .filter(|s| s.part == BodyPart::Move && s.hits > 0)
            .count() as u32;
        if moves == 0 {
            return u32::MAX;
        }
        (total + moves - 1) / moves
    }
}