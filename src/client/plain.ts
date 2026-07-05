import * as readline from 'readline';
import chalk from 'chalk';
import type { MinimapCell, OnlinePlayer, OutputStyle, PlayerSnapshot } from '../protocol/messages.js';

const STYLES: Record<OutputStyle, (s: string) => string> = {
  normal: (s) => s,
  system: chalk.cyan,
  combat: chalk.red,
  chat: chalk.yellow,
  quest: chalk.magenta,
  loot: chalk.green,
  death: chalk.red.bold,
  party: chalk.blue,
  trade: chalk.hex('#FFA500'),
  global: chalk.magenta,
  emote: chalk.cyan.italic,
  epic: chalk.magenta.bold,
};

export interface RoomView {
  title: string;
  description: string;
  exits: string;
  entities: string[];
  zone?: string;
  minimap?: MinimapCell[];
  zoneArt?: string;
}

export interface GameClientUI {
  onInput(handler: (line: string) => void): void;
  showBanner(): void;
  log(text: string, style?: OutputStyle): void;
  showRoom(room: RoomView): void;
  showStats(player: PlayerSnapshot): void;
  showOnline(players: OnlinePlayer[]): void;
  showTicker(text: string): void;
  showMotd(text: string): void;
  flash(color: 'red' | 'yellow' | 'green'): void;
  bell(): void;
  showClassSelect(): void;
  showError(text: string): void;
  setPrompt(text: string, hidden?: boolean): void;
  showDisconnect(reason: string): void;
  destroy(): void;
}

export function createPlainClient(): GameClientUI {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout, terminal: true });
  let inputHandler: ((line: string) => void) | null = null;

  rl.on('line', (line) => inputHandler?.(line));

  return {
    onInput(handler) {
      inputHandler = handler;
    },

    showBanner() {
      console.log(chalk.bold.cyan(`
╔══════════════════════════════════════════════════════════╗
║              R E A L M   O F   E C H O E S              ║
║         A Classic MMO Text Adventure                     ║
╚══════════════════════════════════════════════════════════╝
`));
    },

    log(text, style = 'normal') {
      const formatter = STYLES[style] ?? STYLES.normal;
      for (const line of text.split('\n')) {
        console.log(formatter(line));
      }
    },

    showRoom(room) {
      console.log();
      if (room.zoneArt) console.log(chalk.dim(room.zoneArt));
      console.log(chalk.bold.white(`[ ${room.title} ]`));
      console.log(chalk.white(room.description));
      if (room.exits) console.log(chalk.dim(`Exits: ${room.exits}`));
      if (room.entities.length > 0) {
        console.log(chalk.dim('Also here:'));
        for (const e of room.entities) {
          const hostile = e.includes('[hostile]');
          console.log(chalk.dim('  ') + (hostile ? chalk.red(`• ${e}`) : chalk.green(`• ${e}`)));
        }
      }
      console.log();
    },

    showStats(player) {
      const hp = bar(player.hp, player.maxHp, chalk.red, chalk.dim.red);
      const mp = bar(player.mp, player.maxMp, chalk.blue, chalk.dim.blue);
      const duel = player.inDuel ? chalk.red(' [DUEL]') : '';
      console.log(
        chalk.dim(` Lv.${player.level} `) + hp + chalk.dim(` ${player.hp}/${player.maxHp} `) +
        mp + chalk.dim(` ${player.mp}/${player.maxMp} `) +
        chalk.yellow(`${player.gold}g`) + duel,
      );
    },

    showOnline(players) {
      console.log(chalk.dim(`-- ${players.length} online --`));
      for (const p of players) {
        console.log(chalk.dim(`  ${p.username} Lv.${p.level} ${p.className} @ ${p.zone}`));
      }
    },

    showTicker(text) {
      console.log(chalk.dim(`› ${text}`));
    },

    showMotd(text) {
      console.log(chalk.cyan(`\n=== MOTD ===\n${text}\n`));
    },

    flash() {},

    bell() {
      process.stdout.write('\x07');
    },

    showClassSelect() {
      console.log(chalk.cyan('\nChoose class: warrior | mage | rogue\n'));
    },

    showError(text) {
      console.log(chalk.red(`Error: ${text}`));
    },

    setPrompt(text) {
      rl.setPrompt(text);
      rl.prompt();
    },

    showDisconnect(reason) {
      console.log(chalk.dim(reason));
    },

    destroy() {
      rl.close();
    },
  };
}

function bar(
  current: number,
  max: number,
  fill: (s: string) => string,
  empty: (s: string) => string,
): string {
  const width = 8;
  const filled = Math.round((current / max) * width);
  return fill('█'.repeat(filled)) + empty('░'.repeat(Math.max(0, width - filled)));
}