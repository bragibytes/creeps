import type { WebSocket } from 'ws';
import type { CombatSnapshot, PlayerSnapshot } from '../protocol/messages.js';
import type { StoredPlayer } from '../db/database.js';
import { ITEMS } from './items.js';
import { CLASSES, xpForLevel } from './types.js';
import type { World } from './world.js';
import { findGuildByMember } from './guilds.js';
import type { QuestProgress } from './quests.js';

export class PlayerSession {
  readonly ws: WebSocket;
  data: StoredPlayer;
  combatTarget: string | null = null;
  pvpTarget: string | null = null;
  inDuel = false;
  partyLeader: string | null = null;
  searchedRooms = new Set<string>();
  authenticated = false;

  get inCombat(): boolean {
    return this.combatTarget !== null || this.pvpTarget !== null;
  }

  clearCombat(): void {
    this.combatTarget = null;
    this.pvpTarget = null;
  }

  constructor(ws: WebSocket, data: StoredPlayer) {
    this.ws = ws;
    this.data = data;
  }

  get username(): string {
    return this.data.username;
  }

  get roomId(): string {
    return this.data.roomId;
  }

  get totalAttack(): number {
    const cls = CLASSES[this.data.className];
    let atk = cls.attack + (this.data.level - 1) * 2;
    if (this.data.equipment.weapon) {
      const weapon = ITEMS[this.data.equipment.weapon];
      if (weapon?.attack) atk += weapon.attack;
    }
    return atk;
  }

  get totalDefense(): number {
    const cls = CLASSES[this.data.className];
    let def = cls.defense + (this.data.level - 1);
    if (this.data.equipment.armor) {
      const armor = ITEMS[this.data.equipment.armor];
      if (armor?.defense) def += armor.defense;
    }
    return def;
  }

  get xpToLevel(): number {
    return xpForLevel(this.data.level);
  }

  addXp(amount: number): string[] {
    const messages: string[] = [];
    this.data.xp += amount;
    messages.push(`You gain ${amount} experience.`);

    while (this.data.xp >= this.xpToLevel) {
      this.data.xp -= this.xpToLevel;
      this.levelUp(messages);
    }
    return messages;
  }

  private levelUp(messages: string[]): void {
    this.data.level++;
    const cls = CLASSES[this.data.className];
    const hpGain = Math.floor(cls.maxHp * 0.15) + 5;
    const mpGain = Math.floor(cls.maxMp * 0.12) + 3;
    this.data.maxHp += hpGain;
    this.data.maxMp += mpGain;
    this.data.hp = this.data.maxHp;
    this.data.mp = this.data.maxMp;
    messages.push(`*** LEVEL UP! You are now level ${this.data.level}! ***`);
    messages.push(`HP +${hpGain}, MP +${mpGain}`);
  }

  addItem(itemId: string): void {
    this.data.inventory.push(itemId);
  }

  removeItem(itemId: string): boolean {
    const idx = this.data.inventory.indexOf(itemId);
    if (idx === -1) return false;
    this.data.inventory.splice(idx, 1);
    return true;
  }

  countItem(itemId: string): number {
    return this.data.inventory.filter((id) => id === itemId).length;
  }

  getActiveQuests(): QuestProgress[] {
    return this.data.quests.filter((q) => q.status === 'active');
  }

  toSnapshot(world?: World): PlayerSnapshot {
    const room = world?.rooms.get(this.data.roomId);
    return {
      username: this.data.username,
      className: this.data.className,
      level: this.data.level,
      hp: this.data.hp,
      maxHp: this.data.maxHp,
      mp: this.data.mp,
      maxMp: this.data.maxMp,
      xp: this.data.xp,
      xpToLevel: this.xpToLevel,
      gold: this.data.gold,
      room: this.data.roomId,
      roomName: room?.name,
      zone: room?.zone,
      inCombat: this.inCombat,
      partyLeader: this.partyLeader ?? undefined,
      inDuel: this.inDuel,
      title: this.data.title ?? undefined,
      guildName: findGuildByMember(this.username)?.name,
    };
  }

  toCombatSnapshot(): CombatSnapshot {
    return {
      inCombat: this.inCombat,
      target: this.pvpTarget ?? this.combatTarget ?? undefined,
    };
  }
}