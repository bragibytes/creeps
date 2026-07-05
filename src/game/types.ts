import type { ClassName } from '../protocol/messages.js';

export type LootRarity = 'common' | 'uncommon' | 'rare' | 'epic';

export interface ItemTemplate {
  id: string;
  name: string;
  description: string;
  type: 'weapon' | 'armor' | 'consumable' | 'quest' | 'misc';
  slot?: 'weapon' | 'armor';
  attack?: number;
  defense?: number;
  heal?: number;
  value: number;
  rarity?: LootRarity;
}

export interface MobTemplate {
  id: string;
  name: string;
  description: string;
  level: number;
  hp: number;
  attack: number;
  defense: number;
  xp: number;
  gold: { min: number; max: number };
  loot?: { itemId: string; chance: number }[];
  hostile: boolean;
  respawnSeconds: number;
  elite?: boolean;
  boss?: boolean;
}

export interface NpcTemplate {
  id: string;
  name: string;
  description: string;
  greeting: string;
  questId?: string;
  shop?: { itemId: string; price: number }[];
  crafts?: string[];
}

export interface RoomTemplate {
  id: string;
  name: string;
  description: string;
  zone: string;
  mapX?: number;
  mapY?: number;
  exits: Partial<Record<Direction, string>>;
  items?: string[];
  mobs?: string[];
  npcs?: string[];
}

export interface QuestTemplate {
  id: string;
  name: string;
  description: string;
  giverNpc: string;
  objectives: QuestObjective[];
  rewards: { xp: number; gold: number; items?: string[] };
}

export interface QuestObjective {
  type: 'kill' | 'collect' | 'visit';
  target: string;
  count: number;
  description: string;
}

export interface CraftRecipe {
  id: string;
  name: string;
  output: string;
  ingredients: Record<string, number>;
  gold: number;
  npcId: string;
}

export type Direction = 'north' | 'south' | 'east' | 'west' | 'up' | 'down';

export const DIRECTION_ALIASES: Record<string, Direction> = {
  n: 'north', north: 'north',
  s: 'south', south: 'south',
  e: 'east', east: 'east',
  w: 'west', west: 'west',
  u: 'up', up: 'up',
  d: 'down', down: 'down',
};

export const HOTKEY_COMMANDS: Record<string, string> = {
  n: 'north', s: 'south', e: 'east', w: 'west', u: 'up', d: 'down',
  l: 'look', i: 'inventory', h: 'help', g: 'global',
};

export const RARITY_COLORS: Record<LootRarity, string> = {
  common: 'white',
  uncommon: 'green',
  rare: 'blue',
  epic: 'magenta',
};

export const LOCKED_EXITS: Record<string, { item: string; message: string }> = {
  crypt_stairs: {
    item: 'ancient_key',
    message: 'The iron door is locked. You need the Ancient Key.',
  },
};

export interface ClassStats {
  name: ClassName;
  displayName: string;
  maxHp: number;
  maxMp: number;
  attack: number;
  defense: number;
  ability: string;
  abilityDamage: number;
  abilityCost: number;
  ability2: string;
  ability2Damage: number;
  ability2Cost: number;
  ability2Level: number;
  description: string;
  art: string[];
}

export const CLASSES: Record<ClassName, ClassStats> = {
  warrior: {
    name: 'warrior',
    displayName: 'Warrior',
    maxHp: 120,
    maxMp: 20,
    attack: 12,
    defense: 8,
    ability: 'Power Strike',
    abilityDamage: 25,
    abilityCost: 10,
    ability2: 'Shield Bash',
    ability2Damage: 20,
    ability2Cost: 15,
    ability2Level: 5,
    description: 'A stalwart fighter with high HP and crushing melee attacks.',
    art: ['  ⚔️🛡️  ', ' /|==|\\ ', '  / \\  '],
  },
  mage: {
    name: 'mage',
    displayName: 'Mage',
    maxHp: 70,
    maxMp: 80,
    attack: 8,
    defense: 3,
    ability: 'Fireball',
    abilityDamage: 35,
    abilityCost: 15,
    ability2: 'Chain Lightning',
    ability2Damage: 45,
    ability2Cost: 25,
    ability2Level: 5,
    description: 'A wielder of arcane fire, devastating from range but fragile.',
    art: ['  🔮✨  ', '  /|\\  ', '  / \\  '],
  },
  rogue: {
    name: 'rogue',
    displayName: 'Rogue',
    maxHp: 90,
    maxMp: 40,
    attack: 14,
    defense: 5,
    ability: 'Backstab',
    abilityDamage: 30,
    abilityCost: 12,
    ability2: 'Smoke Bomb',
    ability2Damage: 25,
    ability2Cost: 18,
    ability2Level: 5,
    description: 'A swift shadow who strikes from the darkness with deadly precision.',
    art: ['  🗡️💨  ', '  /|\\  ', '  / \\  '],
  },
};

export const ZONE_ART: Record<string, string> = {
  Eldermoor: '  🏰⚔️🏰  ',
  'Whispering Woods': '  🌲🌳🌲  ',
  'Ironspine Mountains': '  ⛰️🦅⛰️  ',
  'Crypt of Ash': '  💀⚰️💀  ',
};

export const CRAFT_RECIPES: CraftRecipe[] = [
  {
    id: 'craft_leather',
    name: 'Leather Armor',
    output: 'leather_armor',
    ingredients: { wolf_pelt: 3 },
    gold: 10,
    npcId: 'eldermoor_smith',
  },
  {
    id: 'craft_iron_sword',
    name: 'Iron Sword',
    output: 'iron_sword',
    ingredients: { goblin_ear: 5 },
    gold: 25,
    npcId: 'eldermoor_smith',
  },
];

export function xpForLevel(level: number): number {
  return Math.floor(100 * Math.pow(1.5, level - 1));
}