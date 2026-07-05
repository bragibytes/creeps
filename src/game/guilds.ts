import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

export interface Guild {
  id: string;
  name: string;
  leader: string;
  members: string[];
  createdAt: string;
}

interface GuildStore {
  guilds: Guild[];
}

const DATA_DIR = process.env.DATA_DIR ?? join(dirname(fileURLToPath(import.meta.url)), '../../data');
const STORE_PATH = join(DATA_DIR, 'guilds.json');

let store: GuildStore;

export function initGuilds(): void {
  mkdirSync(DATA_DIR, { recursive: true });
  if (existsSync(STORE_PATH)) {
    store = JSON.parse(readFileSync(STORE_PATH, 'utf-8')) as GuildStore;
  } else {
    store = { guilds: [] };
    persist();
  }
}

function persist(): void {
  writeFileSync(STORE_PATH, JSON.stringify(store, null, 2));
}

export function findGuildByMember(username: string): Guild | null {
  const key = username.toLowerCase();
  return store.guilds.find((g) => g.members.some((m) => m.toLowerCase() === key)) ?? null;
}

export function findGuildByName(name: string): Guild | null {
  const lower = name.toLowerCase();
  return store.guilds.find((g) => g.id === lower || g.name.toLowerCase() === lower) ?? null;
}

export function createGuild(leader: string, name: string): Guild | string {
  if (name.length < 3 || name.length > 20) return 'Guild name must be 3-20 characters.';
  if (!/^[a-zA-Z][a-zA-Z0-9 _-]*$/.test(name)) return 'Invalid guild name.';
  if (findGuildByMember(leader)) return 'You are already in a guild.';
  if (findGuildByName(name)) return 'That guild name is taken.';

  const guild: Guild = {
    id: name.toLowerCase().replace(/\s+/g, '_'),
    name,
    leader: leader.toLowerCase(),
    members: [leader.toLowerCase()],
    createdAt: new Date().toISOString(),
  };
  store.guilds.push(guild);
  persist();
  return guild;
}

export function inviteToGuild(guild: Guild, username: string): void {
  const key = username.toLowerCase();
  if (!guild.members.includes(key)) guild.members.push(key);
  persist();
}

export function leaveGuild(username: string): Guild | null {
  const key = username.toLowerCase();
  const guild = findGuildByMember(username);
  if (!guild) return null;

  guild.members = guild.members.filter((m) => m !== key);
  if (guild.members.length === 0) {
    store.guilds = store.guilds.filter((g) => g.id !== guild.id);
  } else if (guild.leader === key) {
    guild.leader = guild.members[0];
  }
  persist();
  return guild;
}

export function getGuildMembers(guild: Guild): string[] {
  return guild.members;
}

export function isLeader(guild: Guild, username: string): boolean {
  return guild.leader === username.toLowerCase();
}