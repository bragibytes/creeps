//! Screeps-style sandbox simulation — authoritative game logic, no I/O.
//!
//! Pure PvP: no NPC creeps, bandits, or invasions. Other players are the only threat.

pub mod constants;
pub mod creep;
pub mod room;
pub mod snapshot;
pub mod structure;
pub mod tick;
pub mod world;

pub use constants::*;
pub use creep::*;
pub use room::*;
pub use snapshot::*;
pub use structure::*;
pub use tick::*;
pub use world::*;