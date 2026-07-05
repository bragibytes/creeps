---
name: realm-rust
description: >
  Realm of Echoes Rust workspace: realm-protocol, realm-core, realm-server,
  realm-client. Use when working in this repo on game logic, WebSocket server,
  ratatui client, or players.json compatibility.
  Read ~/.grok/rules/rust-mastery.md first, then this file for project specifics.
metadata:
  short-description: "Realm of Echoes Rust workspace conventions"
---

# Realm Rust

Project: **Realm of Echoes** — classic MMO text adventure (100% Rust).

## Workspace map

| Crate | Role |
|-------|------|
| `realm-protocol` | `ClientMessage`, `ServerMessage`, `PlayerSnapshot`, `ClassName` |
| `realm-core` | World, combat, quests, `CommandHandler`, DB (`players.json`), guilds |
| `realm-server` | Axum `/ws`, auth, connection routing, world events, save/backup loops |
| `realm-client` | `realm` binary — ratatui TUI (default) + `--plain` CLI |

## Key paths

- World data: `data/world.json`
- Player/guild persistence: **Postgres** via `DATABASE_URL`
- Client server URL: `REALM_SERVER` in `.env` (auto-loaded)

## Server architecture (`crates/realm-server/src/server.rs`)

- `GameServer` holds single instances of `PartyManager`, `TradeManager`, `DuelManager`
- `process_command()` runs sync with `RefCell<Vec<Delivery>>` callbacks
- `flush_deliveries()` sends after locks dropped
- Maps: `conn_tx`, `conn_user`, `user_conn`

## Client architecture (`crates/realm-client/`)

- `app.rs` — auth state machine, `HOTKEY_COMMANDS`, WS connection, reconnect (max 5)
- `plain.rs` — scrollback mode for non-TTY
- `tui.rs` — ratatui layout: header / log / sidebar / input

## Player install (no Rust required)

```bash
curl -fsSL https://raw.githubusercontent.com/bragibytes/space/main/scripts/install.sh | sh
realm
```

Client auto-discovers production URL via `/config`. Release binaries: tag `v*` → GitHub Actions.

## Dev commands

```bash
cargo run -p realm-server
cargo run -p realm-client
cargo run -p realm-client -- --plain
```

## Adding features

1. Wire types → `realm-protocol` if on the WebSocket
2. Logic → `realm-core` module
3. Server callbacks → `server.rs` delivery enum
4. Client handlers → `plain.rs` + `tui.rs`

## Database

- Migrations: `crates/realm-core/migrations/`
- `init_database()` connects via `DATABASE_URL`, runs migrations
- Railway Postgres plugin injects `DATABASE_URL` automatically

## Do not

- Duplicate `CommandHandler` managers in `GameState`
- Use `blocking_lock()` in async WS handlers
- Start `realm-server` unless Sam asks (runtime-ownership)