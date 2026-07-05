import { ITEMS } from './items.js';
import type { PlayerSession } from './player.js';
import type { PartyManager } from './party.js';
import type { TradeManager } from './trade.js';
import type { DuelManager } from './duel.js';
import { CRAFT_RECIPES } from './types.js';
import { isAdmin } from './admin.js';
import {
  createGuild,
  findGuildByMember,
  findGuildByName,
  inviteToGuild,
  leaveGuild,
  isLeader,
} from './guilds.js';
import type { World } from './world.js';
import type { SendFn } from './commands.js';

export function syncPlayerMeta(
  player: PlayerSession,
  party: PartyManager,
  duel: DuelManager,
  players: Map<string, PlayerSession>,
): void {
  const leaderKey = party.getLeaderKey(player);
  player.partyLeader = leaderKey ? (players.get(leaderKey)?.username ?? null) : null;
  player.inDuel = duel.isInDuel(player.username);
}

export function handlePartyCommand(
  player: PlayerSession,
  args: string[],
  party: PartyManager,
  players: Map<string, PlayerSession>,
  send: SendFn,
  roomNotify: (roomId: string, text: string, exclude?: string) => void,
): void {
  const sub = args[0]?.toLowerCase();
  const rest = args.slice(1).join(' ');

  if (sub === 'invite' && rest) {
    const target = findPlayer(players, player, rest);
    if (!target) { send(player, { type: 'output', text: `No player "${rest}" found.`, style: 'party' }); return; }
    const err = party.invite(player, target);
    if (err) send(player, { type: 'output', text: err, style: 'party' });
    else {
      send(player, { type: 'output', text: `Invited ${target.username} to your party.`, style: 'party' });
      send(target, { type: 'output', text: `${player.username} invites you to a party. Type 'party join ${player.username}'.`, style: 'party' });
    }
    return;
  }

  if (sub === 'join' && rest) {
    const err = party.join(player, rest);
    if (err) send(player, { type: 'output', text: err, style: 'party' });
    else {
      send(player, { type: 'output', text: `You joined the party!`, style: 'party' });
      roomNotify(player.roomId, `${player.username} joins the party.`);
    }
    return;
  }

  if (sub === 'leave') {
    party.leave(player);
    send(player, { type: 'output', text: 'You left the party.', style: 'party' });
    return;
  }

  if (sub === 'say' && rest) {
    const leaderKey = party.getLeaderKey(player) ?? player.username.toLowerCase();
    const names = party.getMemberUsernames(leaderKey, players);
    for (const name of names) {
      const p = players.get(name.toLowerCase());
      if (p) send(p, { type: 'output', text: `[Party] ${player.username}: ${rest}`, style: 'party' });
    }
    return;
  }

  const leaderKey = party.getLeaderKey(player);
  if (!leaderKey) {
    send(player, { type: 'output', text: 'You are not in a party. Use party invite <player>.', style: 'party' });
    return;
  }
  const members = party.getMemberUsernames(leaderKey, players);
  send(player, { type: 'output', text: `Party: ${members.join(', ')}`, style: 'party' });
}

export function handleTradeCommand(
  player: PlayerSession,
  args: string[],
  trade: TradeManager,
  players: Map<string, PlayerSession>,
  send: SendFn,
): void {
  const sub = args[0]?.toLowerCase();
  const rest = args.slice(1).join(' ');
  const key = player.username.toLowerCase();

  if (sub === 'cancel') {
    trade.cancel(key);
    send(player, { type: 'output', text: 'Trade cancelled.', style: 'trade' });
    return;
  }

  if (sub === 'accept' && rest) {
    const result = trade.accept(player.username, rest);
    if (typeof result === 'string') {
      send(player, { type: 'output', text: result, style: 'trade' });
      return;
    }
    const partner = players.get(trade.getPartner(result, key));
    send(player, { type: 'output', text: 'Trade session started!', style: 'trade' });
    if (partner) send(partner, { type: 'output', text: `${player.username} accepted your trade.`, style: 'trade' });
    return;
  }

  if (sub === 'offer') {
    const session = trade.getSession(key);
    if (!session) { send(player, { type: 'output', text: 'Not in a trade.', style: 'trade' }); return; }
    if (args[1] === 'gold' && args[2]) {
      const amount = parseInt(args[2], 10);
      if (isNaN(amount) || amount <= 0 || amount > player.data.gold) {
        send(player, { type: 'output', text: 'Invalid gold amount.', style: 'trade' });
        return;
      }
      trade.offer(key, undefined, amount);
    } else {
      const itemName = args.slice(1).join(' ');
      const itemId = findItemInInventory(player, itemName);
      if (!itemId) { send(player, { type: 'output', text: `Don't have "${itemName}".`, style: 'trade' }); return; }
      player.removeItem(itemId);
      trade.offer(key, itemId);
    }
    send(player, { type: 'output', text: 'Trade offer updated.', style: 'trade' });
    return;
  }

  if (sub === 'confirm') {
    const session = trade.getSession(key);
    if (!session) { send(player, { type: 'output', text: 'Not in a trade.', style: 'trade' }); return; }
    if (!trade.confirm(key)) {
      send(player, { type: 'output', text: 'You confirmed. Waiting for partner...', style: 'trade' });
      return;
    }
    executeTrade(session, players, send);
    trade.cancel(key);
    return;
  }

  if (rest) {
    const target = findPlayer(players, player, rest);
    if (!target) { send(player, { type: 'output', text: `No player "${rest}" online.`, style: 'trade' }); return; }
    const err = trade.request(player.username, target.username);
    if (err) send(player, { type: 'output', text: err, style: 'trade' });
    else {
      send(player, { type: 'output', text: `Trade request sent to ${target.username}.`, style: 'trade' });
      send(target, { type: 'output', text: `${player.username} wants to trade. Type 'trade accept ${player.username}'.`, style: 'trade' });
    }
    return;
  }

  send(player, { type: 'output', text: 'Usage: trade <player> | trade accept <player> | trade offer <item|gold N> | trade confirm | trade cancel', style: 'trade' });
}

function executeTrade(
  session: import('./trade.js').TradeSession,
  players: Map<string, PlayerSession>,
  send: SendFn,
): void {
  const a = players.get(session.a);
  const b = players.get(session.b);
  if (!a || !b) return;
  const offerA = session.offers[session.a];
  const offerB = session.offers[session.b];
  a.data.gold -= offerA.gold;
  b.data.gold -= offerB.gold;
  for (const item of offerB.items) a.addItem(item);
  for (const item of offerA.items) b.addItem(item);
  a.data.gold += offerB.gold;
  b.data.gold += offerA.gold;
  send(a, { type: 'output', text: 'Trade complete!', style: 'trade' });
  send(b, { type: 'output', text: 'Trade complete!', style: 'trade' });
}

export function handleDuelCommand(
  player: PlayerSession,
  args: string[],
  duel: DuelManager,
  players: Map<string, PlayerSession>,
  send: SendFn,
  roomNotify: (roomId: string, text: string, exclude?: string) => void,
): void {
  const sub = args[0]?.toLowerCase();
  const rest = args.slice(1).join(' ');

  if (sub === 'accept' && rest) {
    const err = duel.accept(player.username, rest);
    if (err) send(player, { type: 'output', text: err, style: 'combat' });
    else {
      const challenger = players.get(rest.toLowerCase());
      player.inDuel = true;
      if (challenger) challenger.inDuel = true;
      send(player, { type: 'output', text: `Duel accepted! Fight ${rest}!`, style: 'combat' });
      if (challenger) send(challenger, { type: 'output', text: `${player.username} accepted your duel!`, style: 'combat' });
      roomNotify(player.roomId, `${player.username} and ${rest} begin a duel!`);
    }
    return;
  }

  if (rest) {
    const target = findPlayer(players, player, rest);
    if (!target) { send(player, { type: 'output', text: `No player "${rest}" found.`, style: 'combat' }); return; }
    const err = duel.challenge(player.username, target.username);
    if (err) send(player, { type: 'output', text: err, style: 'combat' });
    else {
      send(player, { type: 'output', text: `You challenge ${target.username} to a duel.`, style: 'combat' });
      send(target, { type: 'output', text: `${player.username} challenges you! Type 'duel accept ${player.username}'.`, style: 'combat' });
    }
    return;
  }

  send(player, { type: 'output', text: 'Usage: duel <player> | duel accept <player>', style: 'combat' });
}

export function handleCraftCommand(
  player: PlayerSession,
  recipeId: string,
  world: World,
  send: SendFn,
): void {
  const recipe = CRAFT_RECIPES.find((r) => r.id === recipeId || r.output === recipeId);
  if (!recipe) {
    const list = CRAFT_RECIPES.map((r) => `  ${r.id} — ${r.name}`).join('\n');
    send(player, { type: 'output', text: `Recipes:\n${list}\nUsage: craft <recipe_id>`, style: 'system' });
    return;
  }

  const room = world.getRoom(player.roomId);
  if (!room?.npcs.includes(recipe.npcId)) {
    send(player, { type: 'output', text: 'You must be at the smith to craft.', style: 'system' });
    return;
  }

  for (const [itemId, count] of Object.entries(recipe.ingredients)) {
    if (player.countItem(itemId) < count) {
      send(player, { type: 'output', text: `Need ${count}x ${ITEMS[itemId]?.name ?? itemId}.`, style: 'system' });
      return;
    }
  }
  if (player.data.gold < recipe.gold) {
    send(player, { type: 'output', text: `Need ${recipe.gold} gold.`, style: 'system' });
    return;
  }

  for (const [itemId, count] of Object.entries(recipe.ingredients)) {
    for (let i = 0; i < count; i++) player.removeItem(itemId);
  }
  player.data.gold -= recipe.gold;
  player.addItem(recipe.output);
  send(player, { type: 'output', text: `Crafted ${ITEMS[recipe.output]?.name ?? recipe.output}!`, style: 'loot' });
}

export function handleAdminCommand(
  player: PlayerSession,
  args: string[],
  world: World,
  players: Map<string, PlayerSession>,
  send: SendFn,
  broadcastOnline: () => void,
): void {
  if (!isAdmin(player.username)) {
    send(player, { type: 'output', text: 'Unknown command.', style: 'system' });
    return;
  }

  const sub = args[0]?.toLowerCase();
  const rest = args.slice(1);

  switch (sub) {
    case 'teleport':
      if (rest[0]) {
        player.data.roomId = rest[0];
        send(player, { type: 'output', text: `Teleported to ${rest[0]}.`, style: 'system' });
        broadcastOnline();
      }
      break;
    case 'spawn': {
      const room = world.getRoom(player.roomId);
      const mobId = rest[0];
      const tmpl = world.mobs.get(mobId ?? '');
      if (room && tmpl) {
        room.mobs.push({ instanceId: `admin_${Date.now()}`, templateId: mobId!, hp: tmpl.hp, maxHp: tmpl.hp });
        send(player, { type: 'output', text: `Spawned ${tmpl.name}.`, style: 'system' });
      }
      break;
    }
    case 'setlevel': {
      const lvl = parseInt(rest[0] ?? '', 10);
      if (lvl > 0) {
        player.data.level = lvl;
        send(player, { type: 'output', text: `Level set to ${lvl}.`, style: 'system' });
      }
      break;
    }
    case 'reload': {
      const logs = world.reload();
      send(player, { type: 'output', text: `World reloaded:\n${logs.join('\n')}`, style: 'system' });
      break;
    }
    default:
      send(player, { type: 'output', text: 'Admin: teleport <room> | spawn <mob> | setlevel <n> | reload', style: 'system' });
  }
}

export function handleGuildCommand(
  player: PlayerSession,
  args: string[],
  players: Map<string, PlayerSession>,
  send: SendFn,
  globalGuildChat: (guildId: string, text: string, exclude?: string) => void,
): void {
  const sub = args[0]?.toLowerCase();
  const rest = args.slice(1).join(' ');

  if (sub === 'create' && rest) {
    const result = createGuild(player.username, rest);
    if (typeof result === 'string') {
      send(player, { type: 'output', text: result, style: 'party' });
      return;
    }
    player.data.guildId = result.id;
    send(player, { type: 'output', text: `Guild "${result.name}" founded!`, style: 'party' });
    return;
  }

  if (sub === 'invite' && rest) {
    const guild = findGuildByMember(player.username);
    if (!guild || !isLeader(guild, player.username)) {
      send(player, { type: 'output', text: 'Only guild leaders can invite.', style: 'party' });
      return;
    }
    const target = findPlayer(players, player, rest);
    if (!target) {
      send(player, { type: 'output', text: `No player "${rest}" found.`, style: 'party' });
      return;
    }
    inviteToGuild(guild, target.username);
    target.data.guildId = guild.id;
    send(player, { type: 'output', text: `Invited ${target.username} to ${guild.name}.`, style: 'party' });
    send(target, { type: 'output', text: `You joined guild ${guild.name}!`, style: 'party' });
    return;
  }

  if (sub === 'leave') {
    const guild = leaveGuild(player.username);
    player.data.guildId = null;
    send(player, { type: 'output', text: guild ? `You left ${guild.name}.` : 'You left the guild.', style: 'party' });
    return;
  }

  if (sub === 'say' && rest) {
    const guild = findGuildByMember(player.username);
    if (!guild) {
      send(player, { type: 'output', text: 'You are not in a guild.', style: 'party' });
      return;
    }
    globalGuildChat(guild.id, `[${guild.name}] ${player.username}: ${rest}`);
    return;
  }

  const guild = findGuildByMember(player.username) ?? (sub ? findGuildByName(sub) : null);
  if (!guild) {
    send(player, { type: 'output', text: 'No guild. Visit the Guild Hall and: guild create <name>', style: 'party' });
    return;
  }
  const members = guild.members.map((m) => players.get(m)?.username ?? m).join(', ');
  send(player, {
    type: 'output',
    text: `[${guild.name}] Leader: ${guild.leader}\nMembers: ${members}`,
    style: 'party',
  });
}

function findPlayer(
  players: Map<string, PlayerSession>,
  self: PlayerSession,
  name: string,
): PlayerSession | undefined {
  const lower = name.toLowerCase();
  for (const p of players.values()) {
    if (p.authenticated && p.username.toLowerCase().includes(lower) && p.username !== self.username) {
      return p;
    }
  }
  return undefined;
}

function findItemInInventory(player: PlayerSession, name: string): string | undefined {
  const lower = name.toLowerCase();
  return player.data.inventory.find((id) => {
    const item = ITEMS[id];
    return id.includes(lower) || item?.name.toLowerCase().includes(lower);
  });
}