# Realm of Echoes

A classic MMO text adventure for the terminal. Multiple players share a persistent fantasy world — explore zones, fight monsters, complete quests, trade, party up, and duel.

## Quick Start

```bash
npm install

# Terminal 1 — game server
npm run server

# Terminal 2+ — players (full-screen UI)
npm run client
npm run client:plain    # simple scrollback mode
```

Type `register` to create a character (**warrior**, **mage**, or **rogue**), or `login` to return.

## Client UI

The default client is a **full-screen terminal game interface**:

- Persistent **HP / MP / XP** status bar
- Scrollable **world log** with color-coded messages
- **Location sidebar** — room info, entities, zone map, online players
- **Movement hotkeys** — `n` `s` `e` `w` `l` `i` `h` (no need to type full commands)
- **Combat flash** — status bar pulses on hits
- **Auto-reconnect** — up to 5 retries if connection drops

## Commands

| Category | Commands |
|----------|----------|
| Movement | `north/south/east/west` or `n/s/e/w` |
| Look | `look` / `l` |
| Combat | `attack <target>`, `ability`, `special` (Lv.5+) |
| PvP | Open PvP outside town; `duel <player>`, `duel accept <player>` |
| Items | `get`, `drop`, `inventory`, `equip`, `use`, `buy`, `craft` |
| Social | `say`, `yell`, `whisper`, `party invite/join/leave/say`, `p <msg>` |
| Trade | `trade <player>`, `trade accept`, `trade offer`, `trade confirm` |
| Quests | `talk <npc>`, `accept <id>`, `complete <id>`, `quest` |
| Info | `stats`, `who`, `help`, `rest`, `quit` |

## World

| Zone | Content |
|------|---------|
| **Eldermoor** | Safe town — shops, tavern, smith, quests (PvP disabled) |
| **Whispering Woods** | Goblins, wolves, goblin chief, shrine vault |
| **Ironspine Mountains** | Bandits, cave troll, crystal golem (east from North Gate) |

## Multiplayer Features

- **Parties** — `party invite <player>`, shared XP in the same room
- **Trading** — secure two-player item/gold exchange
- **Duels** — consented PvP (works even in town once accepted)
- **Online list** — sidebar shows who's playing and where

## Crafting

Visit **Greta the Smith** at the North Gate:

```
craft                    # list recipes
craft craft_leather      # wolf pelts → leather armor
craft craft_iron_sword   # goblin ears → iron sword
```

## Deploy (Railway)

```bash
# Set env vars in Railway dashboard:
# PORT=4242
# ADMIN_USERS=your_username

# Players connect with:
REALM_SERVER=wss://your-app.up.railway.app npm run client
```

Mount a **volume** at `/app/data` for persistent player saves.

## Admin

Set `ADMIN_USERS=yourname` in `.env`, then:

```
admin teleport <room_id>
admin spawn <mob_id>
admin setlevel <n>
admin reload
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `4242` | Server WebSocket port |
| `REALM_SERVER` | `ws://localhost:4242` | Client connection URL |
| `ADMIN_USERS` | — | Comma-separated admin usernames |
| `REALM_PLAIN` | — | Force plain client mode |

## Development

```bash
npm run build
npm run dev:server     # hot reload
npm run dev:client
```

Player data saves to `data/players.json` (auto-backup every 30 min in `data/backups/`).
World changes in `src/data/world.json` hot-reload on save.