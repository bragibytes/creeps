import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import worldData from '../data/world.json' with { type: 'json' };
import type { MobTemplate, NpcTemplate, QuestTemplate, RoomTemplate } from './types.js';

const WORLD_PATH = fileURLToPath(new URL('../data/world.json', import.meta.url));

export interface LiveMob {
  instanceId: string;
  templateId: string;
  hp: number;
  maxHp: number;
  elite?: boolean;
}

export interface LiveRoom {
  template: RoomTemplate;
  items: string[];
  mobs: LiveMob[];
  npcs: string[];
}

let mobCounter = 0;

export function nextMobId(): string {
  return `mob_${++mobCounter}`;
}

export class World {
  readonly rooms: Map<string, RoomTemplate> = new Map();
  readonly mobs: Map<string, MobTemplate> = new Map();
  readonly npcs: Map<string, NpcTemplate> = new Map();
  readonly quests: Map<string, QuestTemplate> = new Map();
  readonly liveRooms: Map<string, LiveRoom> = new Map();
  readonly respawnTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();

  constructor() {
    for (const room of worldData.rooms as RoomTemplate[]) {
      this.rooms.set(room.id, room);
    }
    for (const mob of worldData.mobs as MobTemplate[]) {
      this.mobs.set(mob.id, mob);
    }
    for (const npc of worldData.npcs as NpcTemplate[]) {
      this.npcs.set(npc.id, npc);
    }
    for (const quest of worldData.quests as QuestTemplate[]) {
      this.quests.set(quest.id, quest);
    }
    this.initLiveRooms();
  }

  private initLiveRooms(): void {
    for (const [id, template] of this.rooms) {
      const liveMobs: LiveMob[] = (template.mobs ?? []).map((mobId) => {
        const tmpl = this.mobs.get(mobId)!;
        const scale = tmpl.elite ? 1.5 : 1;
        const hp = Math.floor(tmpl.hp * scale);
        return { instanceId: nextMobId(), templateId: mobId, hp, maxHp: hp, elite: tmpl.elite };
      });
      this.liveRooms.set(id, {
        template,
        items: [...(template.items ?? [])],
        mobs: liveMobs,
        npcs: [...(template.npcs ?? [])],
      });
    }
  }

  getRoom(roomId: string): LiveRoom | undefined {
    return this.liveRooms.get(roomId);
  }

  removeMob(roomId: string, instanceId: string): LiveMob | undefined {
    const room = this.liveRooms.get(roomId);
    if (!room) return undefined;
    const idx = room.mobs.findIndex((m) => m.instanceId === instanceId);
    if (idx === -1) return undefined;
    const [removed] = room.mobs.splice(idx, 1);
    this.scheduleRespawn(roomId, removed.templateId);
    return removed;
  }

  private scheduleRespawn(roomId: string, templateId: string): void {
    const tmpl = this.mobs.get(templateId);
    if (!tmpl) return;
    const key = `${roomId}:${templateId}`;
    if (this.respawnTimers.has(key)) return;

    const timer = setTimeout(() => {
      this.respawnTimers.delete(key);
      const room = this.liveRooms.get(roomId);
      if (!room) return;
      const count = room.mobs.filter((m) => m.templateId === templateId).length;
      const template = this.rooms.get(roomId);
      const maxCount = (template?.mobs ?? []).filter((id) => id === templateId).length;
      if (count < maxCount) {
        const scale = tmpl.elite ? 1.5 : 1;
        const hp = Math.floor(tmpl.hp * scale);
        room.mobs.push({
          instanceId: nextMobId(),
          templateId,
          hp,
          maxHp: hp,
          elite: tmpl.elite,
        });
      }
    }, tmpl.respawnSeconds * 1000);

    this.respawnTimers.set(key, timer);
  }

  getZone(roomId: string): string {
    return this.rooms.get(roomId)?.zone ?? 'Unknown';
  }

  reload(): string[] {
    const data = JSON.parse(readFileSync(WORLD_PATH, 'utf-8')) as {
      rooms: RoomTemplate[];
      mobs: MobTemplate[];
      npcs: NpcTemplate[];
      quests: QuestTemplate[];
    };
    const logs: string[] = [];

    this.rooms.clear();
    this.mobs.clear();
    this.npcs.clear();
    this.quests.clear();

    for (const room of data.rooms) this.rooms.set(room.id, room);
    for (const mob of data.mobs) this.mobs.set(mob.id, mob);
    for (const npc of data.npcs) this.npcs.set(npc.id, npc);
    for (const quest of data.quests) this.quests.set(quest.id, quest);

    for (const [id, template] of this.rooms) {
      const existing = this.liveRooms.get(id);
      if (existing) {
        existing.template = template;
        existing.npcs = [...(template.npcs ?? [])];
        logs.push(`Updated room: ${template.name}`);
      } else {
        const liveMobs: LiveMob[] = (template.mobs ?? []).map((mobId) => {
          const tmpl = this.mobs.get(mobId)!;
          return { instanceId: nextMobId(), templateId: mobId, hp: tmpl.hp, maxHp: tmpl.hp };
        });
        this.liveRooms.set(id, {
          template,
          items: [...(template.items ?? [])],
          mobs: liveMobs,
          npcs: [...(template.npcs ?? [])],
        });
        logs.push(`Added room: ${template.name}`);
      }
    }

    return logs;
  }
}