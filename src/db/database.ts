import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import type { ClassName } from '../protocol/messages.js';
import type { QuestProgress } from '../game/quests.js';

export interface StoredPlayer {
  id: number;
  username: string;
  passwordHash: string;
  className: ClassName;
  level: number;
  xp: number;
  hp: number;
  maxHp: number;
  mp: number;
  maxMp: number;
  gold: number;
  roomId: string;
  inventory: string[];
  equipment: { weapon?: string; armor?: string };
  quests: QuestProgress[];
}

interface PlayerStore {
  nextId: number;
  players: StoredPlayer[];
}

const DATA_DIR = process.env.DATA_DIR ?? join(dirname(fileURLToPath(import.meta.url)), '../../data');
const STORE_PATH = join(DATA_DIR, 'players.json');

let store: PlayerStore;

export function initDatabase(): void {
  mkdirSync(DATA_DIR, { recursive: true });
  if (existsSync(STORE_PATH)) {
    store = JSON.parse(readFileSync(STORE_PATH, 'utf-8')) as PlayerStore;
  } else {
    store = { nextId: 1, players: [] };
    persist();
  }
}

function persist(): void {
  writeFileSync(STORE_PATH, JSON.stringify(store, null, 2));
}

export function findPlayer(username: string): StoredPlayer | null {
  return store.players.find((p) => p.username.toLowerCase() === username.toLowerCase()) ?? null;
}

export function createPlayer(
  username: string,
  passwordHash: string,
  className: ClassName,
  stats: { maxHp: number; maxMp: number },
): StoredPlayer {
  const player: StoredPlayer = {
    id: store.nextId++,
    username,
    passwordHash,
    className,
    level: 1,
    xp: 0,
    hp: stats.maxHp,
    maxHp: stats.maxHp,
    mp: stats.maxMp,
    maxMp: stats.maxMp,
    gold: 10,
    roomId: 'eldermoor_square',
    inventory: [],
    equipment: {},
    quests: [],
  };
  store.players.push(player);
  persist();
  return player;
}

export function savePlayer(player: StoredPlayer): void {
  const idx = store.players.findIndex((p) => p.id === player.id);
  if (idx !== -1) {
    store.players[idx] = player;
    persist();
  }
}

export function hashPassword(password: string): string {
  let hash = 0;
  for (let i = 0; i < password.length; i++) {
    hash = ((hash << 5) - hash + password.charCodeAt(i)) | 0;
  }
  return `h${hash.toString(36)}`;
}

export function verifyPassword(password: string, hash: string): boolean {
  return hashPassword(password) === hash;
}