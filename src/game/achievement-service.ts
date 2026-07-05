import type { PlayerSession } from './player.js';
import { ACHIEVEMENTS, grantAchievement } from './achievements.js';
import type { SendFn } from './commands.js';

export function checkAchievements(
  player: PlayerSession,
  send: SendFn,
  triggers: {
    kill?: string;
    room?: string;
    gold?: number;
    level?: number;
    duelWin?: boolean;
  },
): void {
  const earned = player.data.achievements;

  if (triggers.kill && player.data.kills === 1) {
    notify(player, send, grantAchievement(earned, 'first_blood'));
  }

  if (triggers.kill?.includes('goblin') && (player.data.goblinKills ?? 0) >= 10) {
    notify(player, send, grantAchievement(earned, 'goblin_hunter'));
  }

  if (triggers.kill === 'goblin_chief') {
    notify(player, send, grantAchievement(earned, 'chief_slayer'));
  }

  if (triggers.room === 'crystal_cavern') {
    notify(player, send, grantAchievement(earned, 'explorer'));
  }

  if (triggers.gold !== undefined && triggers.gold >= 500) {
    notify(player, send, grantAchievement(earned, 'wealthy'));
  }

  if (triggers.level !== undefined && triggers.level >= 10) {
    notify(player, send, grantAchievement(earned, 'veteran'));
  }

  if (triggers.duelWin) {
    notify(player, send, grantAchievement(earned, 'duelist'));
  }
}

function notify(
  player: PlayerSession,
  send: SendFn,
  achievement: ReturnType<typeof grantAchievement>,
): void {
  if (!achievement) return;
  send(player, {
    type: 'output',
    text: `*** ACHIEVEMENT: ${achievement.name} ***\n${achievement.description}`,
    style: 'quest',
  });
  if (achievement.title) {
    player.data.title = achievement.title;
    send(player, { type: 'output', text: `Title earned: ${achievement.title}`, style: 'loot' });
  }
  send(player, { type: 'bell' });
}

export function formatAchievements(player: PlayerSession): string {
  if (player.data.achievements.length === 0) {
    return 'No achievements yet. Explore, fight, and conquer!';
  }
  const lines = ['-- Achievements --'];
  for (const id of player.data.achievements) {
    const a = ACHIEVEMENTS[id];
    if (a) lines.push(`  ✓ ${a.name} — ${a.description}`);
  }
  if (player.data.title) lines.push(`\nTitle: ${player.data.title}`);
  return lines.join('\n');
}