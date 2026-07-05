import { readFileSync, existsSync, watch } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { WebSocketServer, WebSocket } from 'ws';
import type { ClientMessage, OnlinePlayer, OutputStyle, ServerMessage } from '../protocol/messages.js';
import { initDatabase, findPlayer, createPlayer, savePlayer, hashPassword, verifyPassword } from '../db/database.js';
import { backupPlayersJson } from '../db/backup.js';
import { initGuilds, findGuildByMember } from '../game/guilds.js';
import { World } from '../game/world.js';
import { PlayerSession } from '../game/player.js';
import { CommandHandler } from '../game/commands.js';
import { PartyManager } from '../game/party.js';
import { TradeManager } from '../game/trade.js';
import { DuelManager } from '../game/duel.js';
import { WorldEventManager } from '../game/events.js';
import { CLASSES } from '../game/types.js';

const WORLD_PATH = fileURLToPath(new URL('../data/world.json', import.meta.url));
const DATA_DIR = process.env.DATA_DIR ?? fileURLToPath(new URL('../../data', import.meta.url));
const STORE_PATH = join(DATA_DIR, 'players.json');
const MOTD = process.env.MOTD ?? (existsSync(join(DATA_DIR, 'motd.txt'))
  ? readFileSync(join(DATA_DIR, 'motd.txt'), 'utf-8').trim()
  : 'Welcome to the Realm of Echoes! Type help for commands.');

export class GameServer {
  private wss: WebSocketServer;
  private world = new World();
  private players = new Map<string, PlayerSession>();
  private sessions = new Map<WebSocket, PlayerSession>();
  private party = new PartyManager();
  private trade = new TradeManager();
  private duel = new DuelManager();
  private events: WorldEventManager;
  private commands: CommandHandler;
  private saveInterval: ReturnType<typeof setInterval>;
  private backupInterval: ReturnType<typeof setInterval>;

  constructor(port: number) {
    initDatabase();
    initGuilds();
    this.wss = new WebSocketServer({ port, host: '0.0.0.0' });
    this.events = new WorldEventManager(this.world, (text) => this.globalBroadcast(text, 'global'));
    this.commands = new CommandHandler(
      this.world,
      this.players,
      this.party,
      this.trade,
      this.duel,
      (p, msg) => this.send(p, msg),
      (roomId, msg, exclude) => this.broadcast(roomId, msg, exclude),
      (roomId, text, exclude) => this.roomNotify(roomId, text, exclude),
      () => this.broadcastOnline(),
      (p, color) => this.send(p, { type: 'flash', color }),
      (text, style) => this.globalBroadcast(text, style),
      (text) => this.ticker(text),
      (guildId, text) => this.guildBroadcast(guildId, text),
    );

    this.saveInterval = setInterval(() => this.saveAll(), 30_000);
    this.backupInterval = setInterval(() => backupPlayersJson(DATA_DIR, STORE_PATH), 30 * 60_000);

    try {
      watch(WORLD_PATH, () => {
        const logs = this.world.reload();
        console.log(`World reloaded: ${logs.length} changes`);
      });
    } catch {
      // non-fatal
    }

    this.wss.on('connection', (ws) => this.onConnect(ws));
    console.log(`Realm of Echoes server listening on ws://0.0.0.0:${port}`);
  }

  private onConnect(ws: WebSocket): void {
    this.sendRaw(ws, { type: 'banner' });
    this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data.toString()) as ClientMessage;
        this.handleMessage(ws, msg);
      } catch {
        this.sendRaw(ws, { type: 'error', text: 'Invalid message format.' });
      }
    });

    ws.on('close', () => this.onDisconnect(ws));
  }

  private handleMessage(ws: WebSocket, msg: ClientMessage): void {
    const session = this.sessions.get(ws);

    if (!session?.authenticated) {
      if (msg.type === 'login') {
        this.handleLogin(ws, msg.username, msg.password);
      } else if (msg.type === 'register') {
        this.handleRegister(ws, msg.username, msg.password, msg.className);
      }
      return;
    }

    if (msg.type === 'command') {
      this.commands.handle(session, msg.input);
      savePlayer(session.data);
    }
  }

  private postLogin(session: PlayerSession, welcome: string): void {
    this.send(session, { type: 'output', text: welcome, style: 'system' });
    this.send(session, { type: 'motd', text: MOTD });
    this.roomNotify(session.roomId, `${session.username} enters the realm.`);
    this.commands.handle(session, 'look');
    this.send(session, { type: 'stats', player: session.toSnapshot(this.world) });
    this.broadcastOnline();
    this.send(session, { type: 'prompt', text: '>' });
  }

  private handleLogin(ws: WebSocket, username: string, password: string): void {
    const stored = findPlayer(username);
    if (!stored || !verifyPassword(password, stored.passwordHash)) {
      this.sendRaw(ws, { type: 'error', text: 'Invalid username or password.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }

    if (this.players.has(stored.username.toLowerCase())) {
      this.sendRaw(ws, { type: 'error', text: 'That character is already logged in.' });
      return;
    }

    const session = new PlayerSession(ws, stored);
    session.authenticated = true;
    this.players.set(stored.username.toLowerCase(), session);
    this.sessions.set(ws, session);
    this.postLogin(session, `Welcome back, ${stored.username}!`);
  }

  private handleRegister(
    ws: WebSocket,
    username: string,
    password: string,
    className: 'warrior' | 'mage' | 'rogue',
  ): void {
    if (!username || username.length < 3 || username.length > 16) {
      this.sendRaw(ws, { type: 'error', text: 'Username must be 3-16 characters.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }
    if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(username)) {
      this.sendRaw(ws, { type: 'error', text: 'Username must start with a letter and contain only letters, numbers, underscores.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }
    if (!password || password.length < 4) {
      this.sendRaw(ws, { type: 'error', text: 'Password must be at least 4 characters.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }
    if (!CLASSES[className]) {
      this.sendRaw(ws, { type: 'error', text: 'Choose a class: warrior, mage, or rogue.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }
    if (findPlayer(username)) {
      this.sendRaw(ws, { type: 'error', text: 'Username already taken.' });
      this.sendRaw(ws, { type: 'prompt', text: 'login or register?' });
      return;
    }

    const cls = CLASSES[className];
    const stored = createPlayer(username, hashPassword(password), className, {
      maxHp: cls.maxHp,
      maxMp: cls.maxMp,
    });

    const session = new PlayerSession(ws, stored);
    session.authenticated = true;
    this.players.set(username.toLowerCase(), session);
    this.sessions.set(ws, session);

    this.send(session, {
      type: 'output',
      text: `Character created! You are ${cls.displayName}.\n${cls.description}`,
      style: 'system',
    });
    this.postLogin(session, 'Type "help" for commands. Good luck, adventurer!');
  }

  private onDisconnect(ws: WebSocket): void {
    const session = this.sessions.get(ws);
    if (session) {
      if (session.pvpTarget) {
        const opponent = this.players.get(session.pvpTarget);
        if (opponent) {
          opponent.clearCombat();
          this.send(opponent, { type: 'output', text: `${session.username} fled the battle!`, style: 'combat' });
        }
      }
      this.party.onDisconnect(session);
      this.trade.onDisconnect(session.username);
      this.duel.onDisconnect(session.username);
      savePlayer(session.data);
      this.roomNotify(session.roomId, `${session.username} has left the realm.`);
      this.players.delete(session.username.toLowerCase());
      this.sessions.delete(ws);
      this.broadcastOnline();
    }
  }

  private broadcastOnline(): void {
    const list: OnlinePlayer[] = [...this.players.values()]
      .filter((p) => p.authenticated)
      .map((p) => ({
        username: p.username,
        level: p.data.level,
        className: p.data.className,
        zone: this.world.getZone(p.roomId),
      }));

    for (const p of this.players.values()) {
      if (p.authenticated) {
        this.sendRaw(p.ws, { type: 'online', players: list });
      }
    }
  }

  private globalBroadcast(text: string, style: OutputStyle = 'global'): void {
    for (const p of this.players.values()) {
      if (p.authenticated) {
        this.send(p, { type: 'output', text, style });
      }
    }
  }

  private guildBroadcast(guildId: string, text: string): void {
    for (const p of this.players.values()) {
      if (!p.authenticated) continue;
      const guild = findGuildByMember(p.username);
      if (guild?.id === guildId) {
        this.send(p, { type: 'output', text, style: 'party' });
      }
    }
  }

  private ticker(text: string): void {
    for (const p of this.players.values()) {
      if (p.authenticated) {
        this.sendRaw(p.ws, { type: 'ticker', text });
      }
    }
  }

  private send(player: PlayerSession, msg: ServerMessage): void {
    this.sendRaw(player.ws, msg);
    if (msg.type === 'output' || msg.type === 'error') {
      this.sendRaw(player.ws, { type: 'prompt', text: '>' });
    }
  }

  private sendRaw(ws: WebSocket, msg: ServerMessage): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }

  private broadcast(roomId: string, msg: ServerMessage, exclude?: string): void {
    for (const p of this.players.values()) {
      if (p.roomId === roomId && p.username !== exclude) {
        this.send(p, msg);
      }
    }
  }

  private roomNotify(roomId: string, text: string, exclude?: string): void {
    this.broadcast(roomId, { type: 'output', text, style: 'chat' }, exclude);
  }

  private saveAll(): void {
    for (const session of this.players.values()) {
      savePlayer(session.data);
    }
  }

  shutdown(): void {
    clearInterval(this.saveInterval);
    clearInterval(this.backupInterval);
    this.events.stop();
    this.saveAll();
    backupPlayersJson(DATA_DIR, STORE_PATH);
    this.wss.close();
  }
}