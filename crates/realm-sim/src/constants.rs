/// Tiles per room edge (Screeps uses 50; we start smaller for faster iteration).
pub const ROOM_SIZE: i32 = 25;

/// Simulation ticks per second.
pub const TICKS_PER_SECOND: u32 = 1;

/// Base CPU budget per tick in wasm fuel units (tuned later).
pub const BASE_CPU_PER_TICK: u64 = 1_000_000;

/// Energy capacity of a standard source before regeneration tick.
pub const SOURCE_ENERGY_CAPACITY: i32 = 3000;

/// Energy regenerated per source per 300 ticks in Screeps; simplified for MVP.
pub const SOURCE_ENERGY_REGEN: i32 = 10;

/// Starting GCL — number of rooms a player may claim.
pub const STARTING_GCL: u8 = 1;

/// Controller downgrade hits zero after this many ticks without claim upkeep (simplified).
pub const CONTROLLER_UNCLAIM_TICKS: u64 = 200_000;