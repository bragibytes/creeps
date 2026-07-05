import { ITEMS } from './items.js';
import type { PlayerSession } from './player.js';
import type { LiveMob, World } from './world.js';
import { CLASSES } from './types.js';
import { checkQuestComplete } from './quests.js';

export const SAFE_ZONES = new Set(['Eldermoor']);

export function isSafeZone(world: World, roomId: string): boolean {
  return SAFE_ZONES.has(world.getZone(roomId));
}

export interface CombatResult {
  messages: string[];
  playerDied: boolean;
  mobKilled: boolean;
}

export interface PvpRoundResult {
  attackerMessages: string[];
  defenderMessages: string[];
  defenderDied: boolean;
  attackerDied: boolean;
}

function rollDamage(attack: number, defense: number): number {
  const base = Math.max(1, attack - Math.floor(defense / 2));
  const variance = Math.floor(base * 0.3);
  return base + Math.floor(Math.random() * (variance + 1));
}

export function playerAttack(
  player: PlayerSession,
  mob: LiveMob,
  world: World,
  partyPeers: PlayerSession[] = [],
): CombatResult {
  const tmpl = world.mobs.get(mob.templateId)!;
  const messages: string[] = [];
  const dmg = rollDamage(player.totalAttack, tmpl.defense);
  mob.hp -= dmg;
  messages.push(`You strike ${tmpl.name} for ${dmg} damage! (${mob.hp}/${mob.maxHp} HP)`);

  if (mob.hp <= 0) {
    return handleMobDeath(player, mob, world, messages, partyPeers);
  }

  const counterDmg = rollDamage(tmpl.attack, player.totalDefense);
  player.data.hp -= counterDmg;
  messages.push(`${tmpl.name} hits you for ${counterDmg} damage! (${player.data.hp}/${player.data.maxHp} HP)`);

  if (player.data.hp <= 0) {
    player.data.hp = 0;
    return { messages, playerDied: true, mobKilled: false };
  }

  return { messages, playerDied: false, mobKilled: false };
}

function resolveAbility(player: PlayerSession, slot: 1 | 2) {
  const cls = CLASSES[player.data.className];
  if (slot === 2) {
    if (player.data.level < cls.ability2Level) {
      return { name: cls.ability2, damage: 0, cost: 0, locked: true, level: cls.ability2Level };
    }
    return { name: cls.ability2, damage: cls.ability2Damage, cost: cls.ability2Cost, locked: false, level: cls.ability2Level };
  }
  return { name: cls.ability, damage: cls.abilityDamage, cost: cls.abilityCost, locked: false, level: 1 };
}

export function playerAbility(
  player: PlayerSession,
  mob: LiveMob,
  world: World,
  slot: 1 | 2 = 1,
): CombatResult {
  const ability = resolveAbility(player, slot);
  const messages: string[] = [];

  if (ability.locked) {
    messages.push(`Ability unlocks at level ${ability.level}.`);
    return { messages, playerDied: false, mobKilled: false };
  }

  if (player.data.mp < ability.cost) {
    messages.push(`Not enough MP! ${ability.name} costs ${ability.cost} MP.`);
    return { messages, playerDied: false, mobKilled: false };
  }

  player.data.mp -= ability.cost;
  const tmpl = world.mobs.get(mob.templateId)!;
  const dmg = rollDamage(ability.damage + player.totalAttack, tmpl.defense);
  mob.hp -= dmg;
  messages.push(`*** ${ability.name}! *** You deal ${dmg} damage to ${tmpl.name}! (${mob.hp}/${mob.maxHp} HP)`);

  if (mob.hp <= 0) {
    return handleMobDeath(player, mob, world, messages, []);
  }

  const counterDmg = rollDamage(tmpl.attack, player.totalDefense);
  player.data.hp -= counterDmg;
  messages.push(`${tmpl.name} retaliates for ${counterDmg} damage! (${player.data.hp}/${player.data.maxHp} HP)`);

  if (player.data.hp <= 0) {
    player.data.hp = 0;
    return { messages, playerDied: true, mobKilled: false };
  }

  return { messages, playerDied: false, mobKilled: false };
}

export function playerAbilityVsPlayer(
  attacker: PlayerSession,
  defender: PlayerSession,
  slot: 1 | 2 = 1,
): PvpRoundResult {
  const ability = resolveAbility(attacker, slot);
  const attackerMessages: string[] = [];
  const defenderMessages: string[] = [];

  if (ability.locked) {
    return { attackerMessages: [`Ability unlocks at level ${ability.level}.`], defenderMessages: [], defenderDied: false, attackerDied: false };
  }
  if (attacker.data.mp < ability.cost) {
    return { attackerMessages: [`Not enough MP! ${ability.name} costs ${ability.cost} MP.`], defenderMessages: [], defenderDied: false, attackerDied: false };
  }

  attacker.data.mp -= ability.cost;
  const dmg = rollDamage(ability.damage + attacker.totalAttack, defender.totalDefense);
  defender.data.hp -= dmg;
  attackerMessages.push(`*** ${ability.name}! *** You deal ${dmg} damage to ${defender.username}! (${defender.data.hp}/${defender.data.maxHp} HP)`);
  defenderMessages.push(`*** ${attacker.username} uses ${ability.name}! *** You take ${dmg} damage! (${defender.data.hp}/${defender.data.maxHp} HP)`);

  if (defender.data.hp <= 0) {
    defender.data.hp = 0;
    return { attackerMessages, defenderMessages, defenderDied: true, attackerDied: false };
  }

  const counterDmg = rollDamage(defender.totalAttack, attacker.totalDefense);
  attacker.data.hp -= counterDmg;
  attackerMessages.push(`${defender.username} retaliates for ${counterDmg} damage! (${attacker.data.hp}/${attacker.data.maxHp} HP)`);
  defenderMessages.push(`You retaliate for ${counterDmg} damage! (${attacker.data.hp}/${attacker.data.maxHp} HP)`);

  if (attacker.data.hp <= 0) {
    attacker.data.hp = 0;
    return { attackerMessages, defenderMessages, defenderDied: false, attackerDied: true };
  }

  return { attackerMessages, defenderMessages, defenderDied: false, attackerDied: false };
}

function handleMobDeath(
  player: PlayerSession,
  mob: LiveMob,
  world: World,
  messages: string[],
  partyPeers: PlayerSession[] = [],
): CombatResult {
  const tmpl = world.mobs.get(mob.templateId)!;
  world.removeMob(player.roomId, mob.instanceId);
  player.combatTarget = null;
  player.pvpTarget = null;

  messages.push(`*** ${tmpl.name} has been slain! ***`);
  const gold = tmpl.gold.min + Math.floor(Math.random() * (tmpl.gold.max - tmpl.gold.min + 1));
  if (gold > 0) {
    player.data.gold += gold;
    messages.push(`You loot ${gold} gold.`);
  }

  messages.push(...player.addXp(tmpl.xp));
  if (partyPeers.length > 0) {
    const share = Math.floor(tmpl.xp / partyPeers.length);
    for (const peer of partyPeers) {
      if (peer.username === player.username) continue;
      const peerMsgs = peer.addXp(share);
      messages.push(`[Party] ${peer.username} gains ${share} XP.`);
      void peerMsgs;
    }
  }

  if (tmpl.loot) {
    for (const drop of tmpl.loot) {
      if (Math.random() < drop.chance) {
        player.addItem(drop.itemId);
        messages.push(`You found: ${ITEMS[drop.itemId]?.name ?? drop.itemId}`);
      }
    }
  }

  for (const qp of player.getActiveQuests()) {
    const quest = world.quests.get(qp.questId);
    if (!quest) continue;
    for (const obj of quest.objectives) {
      if (obj.type === 'kill' && obj.target === mob.templateId) {
        qp.progress[obj.target] = (qp.progress[obj.target] ?? 0) + 1;
      }
      if (obj.type === 'collect') {
        qp.progress[obj.target] = player.countItem(obj.target);
      }
    }
    if (checkQuestComplete(quest, qp)) {
      qp.status = 'complete';
      messages.push(`Quest complete: ${quest.name}! Return to the quest giver.`);
    }
  }

  return { messages, playerDied: false, mobKilled: true };
}

export function playerAttackPlayer(
  attacker: PlayerSession,
  defender: PlayerSession,
): PvpRoundResult {
  const attackerMessages: string[] = [];
  const defenderMessages: string[] = [];

  const dmg = rollDamage(attacker.totalAttack, defender.totalDefense);
  defender.data.hp -= dmg;
  attackerMessages.push(
    `You strike ${defender.username} for ${dmg} damage! (${defender.data.hp}/${defender.data.maxHp} HP)`,
  );
  defenderMessages.push(
    `${attacker.username} strikes you for ${dmg} damage! (${defender.data.hp}/${defender.data.maxHp} HP)`,
  );

  if (defender.data.hp <= 0) {
    defender.data.hp = 0;
    return { attackerMessages, defenderMessages, defenderDied: true, attackerDied: false };
  }

  const counterDmg = rollDamage(defender.totalAttack, attacker.totalDefense);
  attacker.data.hp -= counterDmg;
  attackerMessages.push(
    `${defender.username} hits you for ${counterDmg} damage! (${attacker.data.hp}/${attacker.data.maxHp} HP)`,
  );
  defenderMessages.push(
    `You hit ${attacker.username} for ${counterDmg} damage! (${attacker.data.hp}/${attacker.data.maxHp} HP)`,
  );

  if (attacker.data.hp <= 0) {
    attacker.data.hp = 0;
    return { attackerMessages, defenderMessages, defenderDied: false, attackerDied: true };
  }

  return { attackerMessages, defenderMessages, defenderDied: false, attackerDied: false };
}

export function clearPvpBetween(
  a: PlayerSession,
  b: PlayerSession | null,
): void {
  a.clearCombat();
  if (b) b.clearCombat();
}

export function pvpVictoryXp(victimLevel: number): number {
  return Math.max(10, victimLevel * 15);
}

export function handlePlayerDeath(player: PlayerSession, killer?: string): string[] {
  player.clearCombat();
  player.data.hp = Math.floor(player.data.maxHp * 0.5);
  player.data.gold = Math.max(0, player.data.gold - Math.floor(player.data.gold * 0.1));
  player.data.roomId = 'eldermoor_square';
  const lines = [
    killer
      ? `*** You have been slain by ${killer}! ***`
      : '*** You have been defeated! ***',
    'You awaken in Eldermoor Town Square, battered but alive.',
    `You lost some gold. HP restored to ${player.data.hp}.`,
  ];
  return lines;
}