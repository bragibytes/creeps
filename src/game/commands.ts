import { ITEMS } from './items.js';
import {
  playerAbility,
  playerAttack,
  playerAttackPlayer,
  playerAbilityVsPlayer,
  handlePlayerDeath,
  isSafeZone,
  clearPvpBetween,
  pvpVictoryXp,
} from './combat.js';
import { PlayerSession } from './player.js';
import {
  checkQuestComplete,
  createQuestProgress,
  formatQuestProgress,
} from './quests.js';
import { buildMinimap } from './minimap.js';
import { PartyManager } from './party.js';
import { TradeManager } from './trade.js';
import { DuelManager } from './duel.js';
import {
  syncPlayerMeta,
  handlePartyCommand,
  handleTradeCommand,
  handleDuelCommand,
  handleCraftCommand,
  handleAdminCommand,
} from './social.js';
import { CLASSES, DIRECTION_ALIASES, ZONE_ART } from './types.js';
import type { LiveMob, World } from './world.js';
import type { ServerMessage } from '../protocol/messages.js';

export type BroadcastFn = (roomId: string, msg: ServerMessage, exclude?: string) => void;
export type SendFn = (player: PlayerSession, msg: ServerMessage) => void;
export type RoomNotifyFn = (roomId: string, text: string, exclude?: string) => void;
export type FlashFn = (player: PlayerSession, color: 'red' | 'yellow' | 'green') => void;

export class CommandHandler {
  constructor(
    private world: World,
    private players: Map<string, PlayerSession>,
    private party: PartyManager,
    private trade: TradeManager,
    private duel: DuelManager,
    private send: SendFn,
    private broadcast: BroadcastFn,
    private roomNotify: RoomNotifyFn,
    private broadcastOnline: () => void,
    private flash: FlashFn,
  ) {}

  handle(player: PlayerSession, input: string): void {
    const trimmed = input.trim();
    if (!trimmed) return;

    const [cmd, ...args] = trimmed.toLowerCase().split(/\s+/);
    const argStr = args.join(' ');

    const handlers: Record<string, () => void> = {
      look: () => this.look(player),
      l: () => this.look(player),
      north: () => this.move(player, 'north'),
      south: () => this.move(player, 'south'),
      east: () => this.move(player, 'east'),
      west: () => this.move(player, 'west'),
      up: () => this.move(player, 'up'),
      down: () => this.move(player, 'down'),
      n: () => this.move(player, 'north'),
      s: () => this.move(player, 'south'),
      e: () => this.move(player, 'east'),
      w: () => this.move(player, 'west'),
      u: () => this.move(player, 'up'),
      d: () => this.move(player, 'down'),
      say: () => this.say(player, argStr),
      yell: () => this.yell(player, argStr),
      whisper: () => this.whisper(player, args[0], args.slice(1).join(' ')),
      tell: () => this.whisper(player, args[0], args.slice(1).join(' ')),
      attack: () => this.attack(player, argStr),
      kill: () => this.attack(player, argStr),
      ability: () => this.useAbility(player, 1),
      cast: () => this.useAbility(player, 1),
      special: () => this.useAbility(player, 2),
      party: () => handlePartyCommand(player, args, this.party, this.players, this.send, this.roomNotify),
      p: () => args.length > 0
        ? handlePartyCommand(player, ['say', ...args], this.party, this.players, this.send, this.roomNotify)
        : handlePartyCommand(player, [], this.party, this.players, this.send, this.roomNotify),
      trade: () => handleTradeCommand(player, args, this.trade, this.players, this.send),
      duel: () => handleDuelCommand(player, args, this.duel, this.players, this.send, this.roomNotify),
      craft: () => handleCraftCommand(player, argStr, this.world, this.send),
      admin: () => handleAdminCommand(player, args, this.world, this.players, this.send, this.broadcastOnline),
      get: () => this.getItem(player, argStr),
      take: () => this.getItem(player, argStr),
      drop: () => this.dropItem(player, argStr),
      inventory: () => this.showInventory(player),
      inv: () => this.showInventory(player),
      i: () => this.showInventory(player),
      equip: () => this.equip(player, argStr),
      use: () => this.useItem(player, argStr),
      stats: () => this.showStats(player),
      score: () => this.showStats(player),
      who: () => this.who(player),
      quest: () => this.showQuests(player),
      quests: () => this.showQuests(player),
      talk: () => this.talk(player, argStr),
      greet: () => this.talk(player, argStr),
      accept: () => this.acceptQuest(player, argStr),
      complete: () => this.completeQuest(player, argStr),
      turnin: () => this.completeQuest(player, argStr),
      buy: () => this.buy(player, argStr),
      rest: () => this.rest(player),
      help: () => this.help(player),
      quit: () => this.quit(player),
      exit: () => this.quit(player),
    };

    const handler = handlers[cmd];
    if (handler) {
      handler();
      this.finalize(player);
    } else if (DIRECTION_ALIASES[cmd]) {
      this.move(player, DIRECTION_ALIASES[cmd]);
      this.finalize(player);
    } else {
      this.send(player, { type: 'output', text: `Unknown command: "${cmd}". Type 'help' for commands.`, style: 'system' });
      this.finalize(player);
    }
  }

  private finalize(player: PlayerSession): void {
    syncPlayerMeta(player, this.party, this.duel, this.players);
  }

  private partyPeersInRoom(player: PlayerSession): PlayerSession[] {
    return this.party
      .getPartyPeers(player, this.players)
      .filter((p) => p.roomId === player.roomId && p.username !== player.username);
  }

  private look(player: PlayerSession): void {
    const room = this.world.getRoom(player.roomId);
    if (!room) return;

    const exits = Object.keys(room.template.exits).join(', ');
    const entities: string[] = [];

    for (const mob of room.mobs) {
      const tmpl = this.world.mobs.get(mob.templateId)!;
      entities.push(`${tmpl.name} (Lv.${tmpl.level}) [hostile]`);
    }
    for (const npcId of room.npcs) {
      const npc = this.world.npcs.get(npcId)!;
      entities.push(npc.name);
    }
    for (const p of this.players.values()) {
      if (p.roomId === player.roomId && p.username !== player.username) {
        entities.push(`${p.username} (Lv.${p.data.level} ${CLASSES[p.data.className].displayName})`);
      }
    }
    for (const itemId of room.items) {
      entities.push(ITEMS[itemId]?.name ?? itemId);
    }

    const zone = room.template.zone;
    this.send(player, {
      type: 'room',
      title: room.template.name,
      description: room.template.description,
      exits,
      entities,
      zone,
      minimap: buildMinimap(this.world, player.roomId),
      zoneArt: ZONE_ART[zone],
    });
  }

  private move(player: PlayerSession, direction: string): void {
    if (player.inCombat) {
      this.send(player, { type: 'output', text: 'You cannot flee while in combat! Use "attack" or defeat your foe.', style: 'combat' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    if (!room) return;

    const destId = room.template.exits[direction as keyof typeof room.template.exits];
    if (!destId) {
      this.send(player, { type: 'output', text: `You cannot go ${direction}.`, style: 'system' });
      return;
    }

    const dest = this.world.getRoom(destId);
    if (!dest) return;

    const oldRoom = player.roomId;
    const oldZone = this.world.getZone(oldRoom);
    player.data.roomId = destId;
    this.roomNotify(oldRoom, `${player.username} heads ${direction}.`, player.username);
    this.roomNotify(destId, `${player.username} arrives from the ${oppositeDir(direction)}.`, player.username);
    if (this.world.getZone(destId) !== oldZone) this.broadcastOnline();

    for (const qp of player.getActiveQuests()) {
      const quest = this.world.quests.get(qp.questId);
      if (!quest) continue;
      for (const obj of quest.objectives) {
        if (obj.type === 'visit' && obj.target === destId) {
          qp.progress[obj.target] = 1;
          if (checkQuestComplete(quest, qp)) {
            qp.status = 'complete';
            this.send(player, { type: 'output', text: `Quest complete: ${quest.name}! Return to the quest giver.`, style: 'quest' });
          }
        }
      }
    }

    this.look(player);
  }

  private say(player: PlayerSession, message: string): void {
    if (!message) {
      this.send(player, { type: 'output', text: 'Say what? Usage: say <message>', style: 'system' });
      return;
    }
    this.roomNotify(player.roomId, `${player.username} says: "${message}"`);
  }

  private yell(player: PlayerSession, message: string): void {
    if (!message) {
      this.send(player, { type: 'output', text: 'Yell what? Usage: yell <message>', style: 'system' });
      return;
    }
    const zone = this.world.getZone(player.roomId);
    for (const p of this.players.values()) {
      if (this.world.getZone(p.roomId) === zone) {
        this.send(p, { type: 'output', text: `[${zone}] ${player.username} yells: "${message}"`, style: 'chat' });
      }
    }
  }

  private whisper(player: PlayerSession, target: string, message: string): void {
    if (!target || !message) {
      this.send(player, { type: 'output', text: 'Usage: whisper <player> <message>', style: 'system' });
      return;
    }
    const targetPlayer = [...this.players.values()].find(
      (p) => p.username.toLowerCase() === target.toLowerCase(),
    );
    if (!targetPlayer) {
      this.send(player, { type: 'output', text: `${target} is not online.`, style: 'system' });
      return;
    }
    this.send(player, { type: 'output', text: `You whisper to ${targetPlayer.username}: "${message}"`, style: 'chat' });
    this.send(targetPlayer, { type: 'output', text: `${player.username} whispers: "${message}"`, style: 'chat' });
  }

  private findMob(player: PlayerSession, name: string): LiveMob | undefined {
    const room = this.world.getRoom(player.roomId);
    if (!room) return undefined;
    const lower = name.toLowerCase();
    return room.mobs.find((m) => {
      const tmpl = this.world.mobs.get(m.templateId)!;
      return tmpl.name.toLowerCase().includes(lower) || m.templateId.includes(lower);
    });
  }

  private findPlayerInRoom(player: PlayerSession, name: string): PlayerSession | undefined {
    const lower = name.toLowerCase();
    for (const p of this.players.values()) {
      if (
        p.authenticated &&
        p.username.toLowerCase() !== player.username.toLowerCase() &&
        p.roomId === player.roomId &&
        p.username.toLowerCase().includes(lower)
      ) {
        return p;
      }
    }
    return undefined;
  }

  private getPvpOpponent(player: PlayerSession): PlayerSession | undefined {
    if (!player.pvpTarget) return undefined;
    return this.players.get(player.pvpTarget);
  }

  private beginPvp(attacker: PlayerSession, defender: PlayerSession): void {
    attacker.pvpTarget = defender.username.toLowerCase();
    defender.pvpTarget = attacker.username.toLowerCase();
    attacker.combatTarget = null;
    defender.combatTarget = null;
  }

  private resolvePvpVictory(winner: PlayerSession, loser: PlayerSession): void {
    const roomId = winner.roomId;
    clearPvpBetween(winner, loser);
    this.duel.endDuel(winner.username, loser.username);
    winner.inDuel = false;
    loser.inDuel = false;

    const deathMsgs = handlePlayerDeath(loser, winner.username);
    for (const msg of deathMsgs) {
      this.send(loser, { type: 'output', text: msg, style: 'death' });
    }
    this.look(loser);
    this.send(loser, { type: 'stats', player: loser.toSnapshot(this.world) });

    this.send(winner, { type: 'output', text: `*** You have slain ${loser.username}! ***`, style: 'combat' });
    for (const msg of winner.addXp(pvpVictoryXp(loser.data.level))) {
      this.send(winner, { type: 'output', text: msg, style: 'combat' });
    }
    this.roomNotify(roomId, `${winner.username} has slain ${loser.username} in PvP!`);
    this.send(winner, { type: 'stats', player: winner.toSnapshot(this.world) });
  }

  private applyPvpRound(
    attacker: PlayerSession,
    defender: PlayerSession,
    result: ReturnType<typeof playerAttackPlayer>,
  ): void {
    for (const msg of result.attackerMessages) {
      this.send(attacker, { type: 'output', text: msg, style: 'combat' });
    }
    for (const msg of result.defenderMessages) {
      this.send(defender, { type: 'output', text: msg, style: 'combat' });
    }
    this.send(attacker, { type: 'stats', player: attacker.toSnapshot(this.world) });
    this.send(defender, { type: 'stats', player: defender.toSnapshot(this.world) });

    if (result.defenderDied) {
      this.resolvePvpVictory(attacker, defender);
    } else if (result.attackerDied) {
      this.resolvePvpVictory(defender, attacker);
    }
  }

  private attackPlayer(player: PlayerSession, defender: PlayerSession): void {
    const duelOpponent = this.duel.getOpponent(player.username);
    const inDuel = player.inDuel || defender.inDuel;

    if (inDuel) {
      if (duelOpponent !== defender.username.toLowerCase()) {
        this.send(player, { type: 'output', text: 'You can only attack your duel opponent.', style: 'combat' });
        return;
      }
    } else if (isSafeZone(this.world, player.roomId)) {
      this.send(player, { type: 'output', text: 'PvP is disabled in town. Venture into the wilds, or challenge someone to a duel.', style: 'system' });
      return;
    }

    if (player.inCombat && player.pvpTarget !== defender.username.toLowerCase()) {
      this.send(player, { type: 'output', text: 'You are already in combat!', style: 'combat' });
      return;
    }

    if (defender.inCombat && defender.pvpTarget !== player.username.toLowerCase()) {
      this.send(player, { type: 'output', text: `${defender.username} is already fighting someone else.`, style: 'combat' });
      return;
    }

    const firstEngagement = !player.pvpTarget;
    this.beginPvp(player, defender);

    if (firstEngagement) {
      this.send(player, { type: 'output', text: `You attack ${defender.username}!`, style: 'combat' });
      this.send(defender, { type: 'output', text: `${player.username} attacks you! Fight back with "attack ${player.username}"!`, style: 'combat' });
      this.roomNotify(player.roomId, `${player.username} attacks ${defender.username}!`);
    }

    this.applyPvpRound(player, defender, playerAttackPlayer(player, defender));
    this.flash(player, 'red');
    this.flash(defender, 'red');
  }

  private attack(player: PlayerSession, target: string): void {
    if (!target) {
      if (player.pvpTarget) {
        const opponent = this.getPvpOpponent(player);
        if (opponent && opponent.roomId === player.roomId) {
          this.applyPvpRound(player, opponent, playerAttackPlayer(player, opponent));
          return;
        }
        player.pvpTarget = null;
      }
      this.send(player, { type: 'output', text: 'Attack what? Usage: attack <target>', style: 'system' });
      return;
    }

    const defender = this.findPlayerInRoom(player, target);
    if (defender) {
      this.attackPlayer(player, defender);
      return;
    }

    if (player.pvpTarget) {
      this.send(player, { type: 'output', text: 'You are in PvP combat! Finish your fight first.', style: 'combat' });
      return;
    }

    const mob = this.findMob(player, target);
    if (!mob) {
      this.send(player, { type: 'output', text: `No "${target}" here to attack.`, style: 'combat' });
      return;
    }

    player.combatTarget = mob.instanceId;
    const tmpl = this.world.mobs.get(mob.templateId)!;
    this.send(player, { type: 'output', text: `You engage ${tmpl.name} in combat!`, style: 'combat' });

    const result = playerAttack(player, mob, this.world, this.partyPeersInRoom(player));
    for (const msg of result.messages) {
      this.send(player, { type: 'output', text: msg, style: 'combat' });
    }
    this.flash(player, 'red');
    for (const peer of this.partyPeersInRoom(player)) {
      this.send(peer, { type: 'stats', player: peer.toSnapshot(this.world) });
    }

    if (result.playerDied) {
      const deathMsgs = handlePlayerDeath(player);
      for (const msg of deathMsgs) {
        this.send(player, { type: 'output', text: msg, style: 'death' });
      }
      this.look(player);
    } else if (result.mobKilled) {
      this.roomNotify(player.roomId, `${player.username} slays ${tmpl.name}!`);
    }

    this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
  }

  private useAbility(player: PlayerSession, slot: 1 | 2): void {
    if (player.pvpTarget) {
      const opponent = this.getPvpOpponent(player);
      if (!opponent || opponent.roomId !== player.roomId) {
        player.pvpTarget = null;
        this.send(player, { type: 'output', text: 'Your opponent is gone.', style: 'combat' });
        return;
      }
      this.applyPvpRound(player, opponent, playerAbilityVsPlayer(player, opponent, slot));
      this.flash(player, 'yellow');
      this.flash(opponent, 'yellow');
      return;
    }

    if (!player.combatTarget) {
      this.send(player, { type: 'output', text: 'You must be in combat to use your ability.', style: 'combat' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    const mob = room?.mobs.find((m) => m.instanceId === player.combatTarget);
    if (!mob) {
      player.combatTarget = null;
      this.send(player, { type: 'output', text: 'Your target is gone.', style: 'combat' });
      return;
    }

    const result = playerAbility(player, mob, this.world, slot);
    for (const msg of result.messages) {
      this.send(player, { type: 'output', text: msg, style: 'combat' });
    }
    this.flash(player, 'yellow');

    if (result.playerDied) {
      const deathMsgs = handlePlayerDeath(player);
      for (const msg of deathMsgs) {
        this.send(player, { type: 'output', text: msg, style: 'death' });
      }
      this.look(player);
    }

    this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
  }

  private getItem(player: PlayerSession, itemName: string): void {
    if (!itemName) {
      this.send(player, { type: 'output', text: 'Get what? Usage: get <item>', style: 'system' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    if (!room) return;

    const lower = itemName.toLowerCase();
    const idx = room.items.findIndex((id) => {
      const item = ITEMS[id];
      return id.includes(lower) || item?.name.toLowerCase().includes(lower);
    });

    if (idx === -1) {
      this.send(player, { type: 'output', text: `No "${itemName}" here.`, style: 'system' });
      return;
    }

    const itemId = room.items.splice(idx, 1)[0];
    player.addItem(itemId);
    const item = ITEMS[itemId];
    this.send(player, { type: 'output', text: `You pick up ${item?.name ?? itemId}.`, style: 'loot' });

    for (const qp of player.getActiveQuests()) {
      const quest = this.world.quests.get(qp.questId);
      if (!quest) continue;
      for (const obj of quest.objectives) {
        if (obj.type === 'collect' && obj.target === itemId) {
          qp.progress[itemId] = player.countItem(itemId);
          if (checkQuestComplete(quest, qp)) {
            qp.status = 'complete';
            this.send(player, { type: 'output', text: `Quest complete: ${quest.name}! Return to the quest giver.`, style: 'quest' });
          }
        }
      }
    }
  }

  private dropItem(player: PlayerSession, itemName: string): void {
    if (!itemName) {
      this.send(player, { type: 'output', text: 'Drop what? Usage: drop <item>', style: 'system' });
      return;
    }

    const lower = itemName.toLowerCase();
    const itemId = player.data.inventory.find((id) => {
      const item = ITEMS[id];
      return id.includes(lower) || item?.name.toLowerCase().includes(lower);
    });

    if (!itemId || !player.removeItem(itemId)) {
      this.send(player, { type: 'output', text: `You don't have "${itemName}".`, style: 'system' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    room?.items.push(itemId);
    const item = ITEMS[itemId];
    this.send(player, { type: 'output', text: `You drop ${item?.name ?? itemId}.`, style: 'system' });
    this.roomNotify(player.roomId, `${player.username} drops ${item?.name ?? itemId}.`, player.username);
  }

  private showInventory(player: PlayerSession): void {
    if (player.data.inventory.length === 0) {
      this.send(player, { type: 'output', text: 'Your inventory is empty.', style: 'system' });
      return;
    }

    const counts = new Map<string, number>();
    for (const id of player.data.inventory) {
      counts.set(id, (counts.get(id) ?? 0) + 1);
    }

    const lines = ['-- Inventory --'];
    for (const [id, count] of counts) {
      const item = ITEMS[id];
      const equipped =
        player.data.equipment.weapon === id || player.data.equipment.armor === id
          ? ' [equipped]'
          : '';
      lines.push(`  ${item?.name ?? id}${count > 1 ? ` x${count}` : ''}${equipped}`);
    }
    lines.push(`Gold: ${player.data.gold}`);
    this.send(player, { type: 'output', text: lines.join('\n'), style: 'system' });
  }

  private equip(player: PlayerSession, itemName: string): void {
    if (!itemName) {
      this.send(player, { type: 'output', text: 'Equip what? Usage: equip <item>', style: 'system' });
      return;
    }

    const lower = itemName.toLowerCase();
    const itemId = player.data.inventory.find((id) => {
      const item = ITEMS[id];
      return id.includes(lower) || item?.name.toLowerCase().includes(lower);
    });

    if (!itemId) {
      this.send(player, { type: 'output', text: `You don't have "${itemName}".`, style: 'system' });
      return;
    }

    const item = ITEMS[itemId];
    if (!item?.slot) {
      this.send(player, { type: 'output', text: `${item?.name ?? itemId} cannot be equipped.`, style: 'system' });
      return;
    }

    player.data.equipment[item.slot] = itemId;
    this.send(player, { type: 'output', text: `You equip ${item.name}.`, style: 'loot' });
  }

  private useItem(player: PlayerSession, itemName: string): void {
    if (!itemName) {
      this.send(player, { type: 'output', text: 'Use what? Usage: use <item>', style: 'system' });
      return;
    }

    const lower = itemName.toLowerCase();
    const itemId = player.data.inventory.find((id) => {
      const item = ITEMS[id];
      return id.includes(lower) || item?.name.toLowerCase().includes(lower);
    });

    if (!itemId) {
      this.send(player, { type: 'output', text: `You don't have "${itemName}".`, style: 'system' });
      return;
    }

    const item = ITEMS[itemId];
    if (item?.type === 'consumable' && item.heal) {
      player.removeItem(itemId);
      const healed = Math.min(item.heal, player.data.maxHp - player.data.hp);
      player.data.hp += healed;
      this.send(player, { type: 'output', text: `You drink ${item.name} and recover ${healed} HP.`, style: 'loot' });
      this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
    } else if (itemId === 'mana_potion') {
      player.removeItem(itemId);
      const restored = Math.min(40, player.data.maxMp - player.data.mp);
      player.data.mp += restored;
      this.send(player, { type: 'output', text: `You drink Mana Potion and recover ${restored} MP.`, style: 'loot' });
      this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
    } else {
      this.send(player, { type: 'output', text: `You can't use ${item?.name ?? itemId} right now.`, style: 'system' });
    }
  }

  private showStats(player: PlayerSession): void {
    const cls = CLASSES[player.data.className];
    const lines = [
      `-- ${player.username} --`,
      `${cls.displayName} | Level ${player.data.level}`,
      `HP: ${player.data.hp}/${player.data.maxHp}  MP: ${player.data.mp}/${player.data.maxMp}`,
      `XP: ${player.data.xp}/${player.xpToLevel}`,
      `Attack: ${player.totalAttack}  Defense: ${player.totalDefense}`,
      `Gold: ${player.data.gold}`,
      `Ability: ${cls.ability} (${cls.abilityCost} MP)`,
    ];
    this.send(player, { type: 'output', text: lines.join('\n'), style: 'system' });
    this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
  }

  private who(player: PlayerSession): void {
    this.broadcastOnline();
  }

  private showQuests(player: PlayerSession): void {
    const active = player.getActiveQuests();
    const complete = player.data.quests.filter((q) => q.status === 'complete');

    if (active.length === 0 && complete.length === 0) {
      this.send(player, { type: 'output', text: 'You have no quests. Talk to NPCs to find work.', style: 'quest' });
      return;
    }

    const lines: string[] = [];
    for (const qp of active) {
      const quest = this.world.quests.get(qp.questId);
      if (quest) lines.push(formatQuestProgress(quest, qp));
    }
    for (const qp of complete) {
      const quest = this.world.quests.get(qp.questId);
      if (quest) lines.push(`[${quest.name}] COMPLETE - return to quest giver!`);
    }
    this.send(player, { type: 'output', text: lines.join('\n\n'), style: 'quest' });
  }

  private talk(player: PlayerSession, npcName: string): void {
    if (!npcName) {
      this.send(player, { type: 'output', text: 'Talk to whom? Usage: talk <npc>', style: 'system' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    if (!room) return;

    const lower = npcName.toLowerCase();
    const npcId = room.npcs.find((id) => {
      const npc = this.world.npcs.get(id)!;
      return id.includes(lower) || npc.name.toLowerCase().includes(lower);
    });

    if (!npcId) {
      this.send(player, { type: 'output', text: `No "${npcName}" here.`, style: 'system' });
      return;
    }

    const npc = this.world.npcs.get(npcId)!;
    this.send(player, { type: 'output', text: `${npc.name}: "${npc.greeting}"`, style: 'chat' });

    if (npc.questId) {
      const quest = this.world.quests.get(npc.questId)!;
      const existing = player.data.quests.find((q) => q.questId === npc.questId);
      if (!existing) {
        this.send(player, { type: 'output', text: `[Quest available: ${quest.name}] Type 'accept ${quest.id}' to begin.`, style: 'quest' });
      }
    }
  }

  private acceptQuest(player: PlayerSession, questId: string): void {
    if (!questId) {
      this.send(player, { type: 'output', text: 'Accept which quest? Usage: accept <quest_id>', style: 'system' });
      return;
    }

    const quest = this.world.quests.get(questId);
    if (!quest) {
      this.send(player, { type: 'output', text: `Unknown quest: ${questId}`, style: 'system' });
      return;
    }

    const existing = player.data.quests.find((q) => q.questId === questId);
    if (existing && existing.status !== 'turned_in') {
      this.send(player, { type: 'output', text: 'You already have that quest.', style: 'quest' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    if (!room?.npcs.includes(quest.giverNpc)) {
      this.send(player, { type: 'output', text: 'You must be near the quest giver to accept this quest.', style: 'quest' });
      return;
    }

    if (existing?.status === 'turned_in') {
      existing.status = 'active';
      existing.progress = createQuestProgress(quest).progress;
    } else {
      player.data.quests.push(createQuestProgress(quest));
    }

    this.send(player, { type: 'output', text: `Quest accepted: ${quest.name}`, style: 'quest' });
    this.send(player, { type: 'output', text: quest.description, style: 'quest' });
  }

  private completeQuest(player: PlayerSession, questId: string): void {
    if (!questId) {
      const completable = player.data.quests.filter((q) => q.status === 'complete');
      if (completable.length === 1) {
        questId = completable[0].questId;
      } else {
        this.send(player, { type: 'output', text: 'Turn in which quest? Usage: complete <quest_id>', style: 'system' });
        return;
      }
    }

    const qp = player.data.quests.find((q) => q.questId === questId);
    if (!qp || qp.status !== 'complete') {
      this.send(player, { type: 'output', text: 'That quest is not ready to turn in.', style: 'quest' });
      return;
    }

    const quest = this.world.quests.get(questId)!;
    const room = this.world.getRoom(player.roomId);
    if (!room?.npcs.includes(quest.giverNpc)) {
      this.send(player, { type: 'output', text: 'You must return to the quest giver.', style: 'quest' });
      return;
    }

    for (const obj of quest.objectives) {
      if (obj.type === 'collect') {
        for (let i = 0; i < obj.count; i++) {
          player.removeItem(obj.target);
        }
      }
    }

    qp.status = 'turned_in';
    player.data.gold += quest.rewards.gold;
    const xpMsgs = player.addXp(quest.rewards.xp);
    if (quest.rewards.items) {
      for (const itemId of quest.rewards.items) {
        player.addItem(itemId);
        this.send(player, { type: 'output', text: `Received: ${ITEMS[itemId]?.name ?? itemId}`, style: 'loot' });
      }
    }

    this.send(player, { type: 'output', text: `Quest turned in: ${quest.name}! +${quest.rewards.gold} gold, +${quest.rewards.xp} XP`, style: 'quest' });
    for (const msg of xpMsgs) {
      this.send(player, { type: 'output', text: msg, style: 'quest' });
    }
    this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
  }

  private buy(player: PlayerSession, itemName: string): void {
    if (!itemName) {
      this.send(player, { type: 'output', text: 'Buy what? Usage: buy <item>', style: 'system' });
      return;
    }

    const room = this.world.getRoom(player.roomId);
    if (!room) return;

    const lower = itemName.toLowerCase().replace(/\s+/g, '_');
    for (const npcId of room.npcs) {
      const npc = this.world.npcs.get(npcId)!;
      if (!npc.shop) continue;
      const listing = npc.shop.find((s) => s.itemId === lower || ITEMS[s.itemId]?.name.toLowerCase().replace(/\s+/g, '_').includes(lower));
      if (listing) {
        if (player.data.gold < listing.price) {
          this.send(player, { type: 'output', text: `Not enough gold. ${ITEMS[listing.itemId]?.name} costs ${listing.price} gold.`, style: 'system' });
          return;
        }
        player.data.gold -= listing.price;
        player.addItem(listing.itemId);
        this.send(player, { type: 'output', text: `You buy ${ITEMS[listing.itemId]?.name} for ${listing.price} gold.`, style: 'loot' });
        this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
        return;
      }
    }

    this.send(player, { type: 'output', text: `Nobody here sells "${itemName}".`, style: 'system' });
  }

  private rest(player: PlayerSession): void {
    if (player.roomId !== 'eldermoor_tavern') {
      this.send(player, { type: 'output', text: 'You can only rest at The Gilded Tankard tavern.', style: 'system' });
      return;
    }
    if (player.inCombat) {
      this.send(player, { type: 'output', text: 'You cannot rest during combat!', style: 'combat' });
      return;
    }

    const healed = Math.min(30, player.data.maxHp - player.data.hp);
    const mpRestored = Math.min(20, player.data.maxMp - player.data.mp);
    player.data.hp += healed;
    player.data.mp += mpRestored;
    this.send(player, { type: 'output', text: `You rest by the hearth. Recovered ${healed} HP and ${mpRestored} MP.`, style: 'system' });
    this.send(player, { type: 'stats', player: player.toSnapshot(this.world) });
  }

  private help(player: PlayerSession): void {
    const cls = CLASSES[player.data.className];
    const text = `
=== REALM OF ECHOES - Commands ===

Movement:     north/south/east/west/up/down (n/s/e/w/u/d)
Look:         look (l)
Combat:       attack <target>, ability, special (Lv.${cls.ability2Level})
              PvP in wilds; duel <player> for consented fights
Items:        get/take <item>, drop <item>, inventory (i)
              equip <item>, use <item>, buy <item>, craft <recipe>
Social:       say/yell/whisper, party invite/join/leave/say, p <msg>
              trade <player>, trade offer/confirm/cancel
Quests:       talk <npc>, accept <quest_id>, complete <quest_id>
Info:         stats, who, help, rest, quit
Hotkeys:      n/s/e/w move, l look, i inv, h help

Abilities:    ${cls.ability} (${cls.abilityCost} MP)
              ${cls.ability2} (${cls.ability2Cost} MP, Lv.${cls.ability2Level})
`.trim();
    this.send(player, { type: 'output', text, style: 'system' });
  }

  private quit(player: PlayerSession): void {
    this.send(player, { type: 'disconnect', reason: 'Farewell, adventurer!' });
    player.ws.close();
  }
}

function oppositeDir(dir: string): string {
  const opposites: Record<string, string> = {
    north: 'south', south: 'north', east: 'west', west: 'east', up: 'down', down: 'up',
  };
  return opposites[dir] ?? dir;
}