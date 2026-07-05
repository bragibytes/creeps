//! # Realm SDK
//!
//! Write colony AI in Rust, compile to WASM, upload to the server.
//!
//! ```text
//! rustup target add wasm32-unknown-unknown
//! cargo build --release --target wasm32-unknown-unknown -p my-colony-ai
//! ```
//!
//! The host calls `realm_tick()` once per simulation tick with a fuel budget.
//! Only sandboxed APIs are available — no filesystem, network, or threads.

/// Opaque handle to the simulation context (implemented by the server host).
pub struct LordContext {
    _private: (),
}

/// Called by the server each tick. Return `false` to yield remaining CPU.
#[no_mangle]
pub extern "C" fn realm_tick() -> bool {
    tick(&mut LordContext { _private: () })
}

/// Player logic entry point.
pub fn tick(_ctx: &mut LordContext) -> bool {
    // Default starter: do nothing (hardcoded harvest AI runs until player uploads code).
    true
}

/// Log a line to the in-game console (host provides implementation).
#[inline(always)]
pub fn log(_msg: &str) {
    // Host links this to ScriptLog messages once WASM imports are wired.
}