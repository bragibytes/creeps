export type ClientMessage =
  | { type: 'login'; username: string; password: string }
  | { type: 'register'; username: string; password: string; className: ClassName }
  | { type: 'command'; input: string };

export type ServerMessage =
  | { type: 'banner' }
  | { type: 'prompt'; text: string }
  | { type: 'output'; text: string; style?: OutputStyle }
  | { type: 'room'; title: string; description: string; exits: string; entities: string[]; zone?: string; minimap?: MinimapCell[]; zoneArt?: string }
  | { type: 'stats'; player: PlayerSnapshot }
  | { type: 'online'; players: OnlinePlayer[] }
  | { type: 'flash'; color: 'red' | 'yellow' | 'green' }
  | { type: 'combat'; state: CombatSnapshot }
  | { type: 'error'; text: string }
  | { type: 'disconnect'; reason: string };

export type OutputStyle = 'normal' | 'system' | 'combat' | 'chat' | 'quest' | 'loot' | 'death' | 'party' | 'trade';

export type ClassName = 'warrior' | 'mage' | 'rogue';

export interface PlayerSnapshot {
  username: string;
  className: ClassName;
  level: number;
  hp: number;
  maxHp: number;
  mp: number;
  maxMp: number;
  xp: number;
  xpToLevel: number;
  gold: number;
  room: string;
  roomName?: string;
  zone?: string;
  inCombat?: boolean;
  partyLeader?: string;
  inDuel?: boolean;
}

export interface OnlinePlayer {
  username: string;
  level: number;
  className: ClassName;
  zone: string;
}

export interface MinimapCell {
  id: string;
  mapX: number;
  mapY: number;
  name: string;
  current: boolean;
  hasExit: boolean;
}

export interface CombatSnapshot {
  inCombat: boolean;
  target?: string;
  targetHp?: number;
  targetMaxHp?: number;
}