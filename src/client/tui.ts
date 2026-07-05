import blessed from 'neo-blessed';
import type { Widgets } from 'blessed';
import type { ClassName, OnlinePlayer, OutputStyle, PlayerSnapshot } from '../protocol/messages.js';
import type { GameClientUI, RoomView } from './plain.js';
import { renderMinimapAscii } from '../game/minimap.js';
import { CLASSES } from '../game/types.js';

const STYLE_TAGS: Record<OutputStyle, string> = {
  normal: 'white',
  system: 'cyan',
  combat: 'red',
  chat: 'yellow',
  quest: 'magenta',
  loot: 'green',
  death: 'red-bold',
  party: 'blue',
  trade: 'yellow',
};

const CLASS_LABELS: Record<ClassName, string> = {
  warrior: 'Warrior',
  mage: 'Mage',
  rogue: 'Rogue',
};

const LOGIN_ART = [
  '{center}{cyan-fg}REALM OF ECHOES{/}',
  '',
  '{center}{white-fg}A Classic MMO{/}',
  '{center}{white-fg}Text Adventure{/}',
  '',
  '{gray-fg}login    — return{/}',
  '{gray-fg}register — new hero{/}',
].join('\n');

const HELP_HINTS = [
  '{gray-fg}n s e w  move{/}',
  '{gray-fg}l look  i inv{/}',
  '{gray-fg}h help  attack{/}',
].join('\n');

export function createTuiClient(): GameClientUI {
  const screen: Widgets.Screen = blessed.screen({
    smartCSR: true,
    fullUnicode: true,
    title: 'Realm of Echoes',
    dockBorders: true,
  });

  const header: Widgets.BoxElement = blessed.box({
    parent: screen,
    top: 0,
    left: 0,
    width: '100%',
    height: 3,
    tags: true,
    border: { type: 'line' },
    style: { border: { fg: 'cyan' }, fg: 'white', bg: 'black' },
    content: '{center}{cyan-fg}Connecting...{/}',
  });

  const sidebar: Widgets.BoxElement = blessed.box({
    parent: screen,
    top: 3,
    right: 0,
    width: '34%',
    bottom: 3,
    label: ' Location ',
    tags: true,
    border: { type: 'line' },
    style: { border: { fg: 'blue' }, fg: 'white', bg: 'black' },
    padding: { left: 1, right: 1 },
    content: LOGIN_ART,
    wrap: true,
    scrollable: true,
    alwaysScroll: true,
  });

  const log: Widgets.Log = blessed.log({
    parent: screen,
    top: 3,
    left: 0,
    width: '66%',
    bottom: 3,
    label: ' World ',
    tags: true,
    border: { type: 'line' },
    style: { border: { fg: 'cyan' }, fg: 'white', bg: 'black' },
    padding: { left: 1, right: 1 },
    scrollable: true,
    alwaysScroll: true,
    scrollbar: { ch: '▐', style: { bg: 'cyan', fg: 'black' } },
    mouse: true,
    keys: true,
    vi: true,
  });

  const input: Widgets.TextboxElement = blessed.textbox({
    parent: screen,
    bottom: 0,
    left: 0,
    width: '100%',
    height: 3,
    label: ' Command ',
    tags: true,
    border: { type: 'line' },
    style: {
      border: { fg: 'yellow' },
      fg: 'white',
      bg: 'black',
      focus: { border: { fg: 'white' } },
    },
    inputOnFocus: true,
    padding: { left: 1 },
  });

  let inputHandler: ((line: string) => void) | null = null;
  let currentStats: PlayerSnapshot | null = null;
  let onlinePlayers: OnlinePlayer[] = [];
  let lastRoom: RoomView | null = null;
  let inCombat = false;

  function renderHeader(): void {
    if (!currentStats) {
      header.setContent('{center}{cyan-fg}REALM OF ECHOES{/}  {gray-fg}awaiting hero...{/}');
      header.style.border = { fg: 'cyan' };
      screen.render();
      return;
    }

    const p = currentStats;
    const cls = CLASS_LABELS[p.className];
    const hp = meter(p.hp, p.maxHp, 14, 'red');
    const mp = meter(p.mp, p.maxMp, 10, 'blue');
    const xp = meter(p.xp, p.xpToLevel, 10, 'green');
    const tags: string[] = [];
    if (inCombat) tags.push('{red-fg}{bold}⚔{/}');
    if (p.inDuel) tags.push('{red-fg}DUEL{/}');
    if (p.partyLeader) tags.push('{blue-fg}PTY{/}');

    header.setContent(
      `{bold}${p.username}{/}  {cyan-fg}Lv.${p.level}{/} {white-fg}${cls}{/}` +
      `{yellow-fg}  ${p.gold}g{/}  ${tags.join(' ')}  {gray-fg}${p.zone ?? ''}{/}\n` +
      `{red-fg}HP{/} ${hp} {gray-fg}${p.hp}/${p.maxHp}{/}  ` +
      `{blue-fg}MP{/} ${mp} {gray-fg}${p.mp}/${p.maxMp}{/}  ` +
      `{green-fg}XP{/} ${xp}`,
    );
    header.style.border = { fg: inCombat ? 'red' : 'cyan' };
    screen.render();
  }

  function renderSidebar(room?: RoomView): void {
    if (room) lastRoom = room;
    const r = lastRoom;
    const lines: string[] = [];

    if (!r) {
      sidebar.setContent(LOGIN_ART);
      screen.render();
      return;
    }

    if (r.zoneArt) lines.push(`{center}${r.zoneArt}{/}`, '');
    lines.push(`{bold}{white-fg}${r.title}{/}`, '', wrapBlessed(r.description, 28));

    if (r.minimap && r.minimap.length > 0) {
      lines.push('', '{cyan-fg}Map{/}', renderMinimapAscii(r.minimap));
    }

    if (r.exits) {
      lines.push('', '{cyan-fg}Exits{/}', formatExits(r.exits));
    }

    if (r.entities.length > 0) {
      lines.push('', '{cyan-fg}Here{/}');
      for (const entity of r.entities) {
        if (entity.includes('[hostile]')) lines.push(`{red-fg}▸ ${entity.replace(' [hostile]', '')}{/}`);
        else if (entity.includes('(Lv.')) lines.push(`{yellow-fg}▸ ${entity}{/}`);
        else lines.push(`{green-fg}▸ ${entity}{/}`);
      }
    }

    if (onlinePlayers.length > 0) {
      lines.push('', '{cyan-fg}Online{/}');
      for (const p of onlinePlayers.slice(0, 8)) {
        lines.push(`{gray-fg}${p.username}{/} {white-fg}L${p.level}{/} {gray-fg}${p.zone}{/}`);
      }
      if (onlinePlayers.length > 8) lines.push(`{gray-fg}+${onlinePlayers.length - 8} more{/}`);
    }

    lines.push('', '─'.repeat(24), HELP_HINTS);
    sidebar.setContent(lines.join('\n'));
    screen.render();
  }

  input.on('submit', (value: string) => {
    const line = value.trim();
    input.clearValue();
    input.setContent('');
    screen.render();
    if (line) inputHandler?.(line);
    input.focus();
  });

  screen.key(['C-c'], () => process.exit(0));

  screen.render();
  input.focus();

  return {
    onInput(handler) {
      inputHandler = handler;
    },

    showBanner() {
      appendLog('{center}{cyan-fg}{bold}REALM OF ECHOES{/}\n{center}{gray-fg}The realm awaits...{/}', 'system');
      sidebar.setContent(LOGIN_ART);
      screen.render();
    },

    log(text, style = 'normal') {
      const prefix = style === 'combat' ? '⚔ ' : '';
      for (const line of text.split('\n')) {
        appendLog(prefix + line, style);
      }
    },

    showRoom(room) {
      renderSidebar(room);
      appendLog(`{bold}{white-fg}▣ ${room.title}{/}`, 'normal');
      for (const line of wrapLines(room.description, 58)) {
        appendLog(line, 'normal');
      }
    },

    showStats(player) {
      currentStats = player;
      inCombat = player.inCombat ?? false;
      renderHeader();
    },

    showOnline(players) {
      onlinePlayers = players;
      renderSidebar();
    },

    flash(color) {
      const border = color === 'red' ? 'red' : color === 'yellow' ? 'yellow' : 'green';
      header.style.border = { fg: border };
      screen.render();
      setTimeout(() => {
        header.style.border = { fg: inCombat ? 'red' : 'cyan' };
        screen.render();
      }, 120);
    },

    showClassSelect() {
      const lines = ['{cyan-fg}Choose your class:{/}', ''];
      for (const cls of Object.values(CLASSES)) {
        lines.push(`{yellow-fg}${cls.name}{/} — ${cls.displayName}`);
        for (const row of cls.art) lines.push(`  ${row}`);
        lines.push(`  {gray-fg}${cls.description}{/}`, '');
      }
      sidebar.setContent(lines.join('\n'));
      screen.render();
    },

    showError(text) {
      appendLog(`{red-fg}✗ ${text}{/}`, 'system');
    },

    setPrompt(text, hidden = false) {
      input.setLabel(` ${text.trim() || '>'} `);
      (input as { censor?: string | boolean }).censor = hidden ? '*' : false;
      input.focus();
      screen.render();
    },

    showDisconnect(reason) {
      appendLog(`{gray-fg}${reason}{/}`, 'system');
      screen.render();
    },

    destroy() {
      screen.destroy();
    },
  };

  function appendLog(text: string, style: OutputStyle): void {
    const tag = STYLE_TAGS[style] ?? 'white';
    const colored = text.includes('{') ? text : `{${tag}-fg}${text}{/}`;
    log.log(colored);
    screen.render();
  }
}

function meter(current: number, max: number, width: number, color: string): string {
  const pct = max > 0 ? current / max : 0;
  const filled = Math.round(pct * width);
  const empty = Math.max(0, width - filled);
  return `{${color}-fg}` + '█'.repeat(filled) + '{/}{gray-fg}' + '░'.repeat(empty) + '{/}';
}

function formatExits(exits: string): string {
  const dirs: Record<string, string> = {
    north: '↑n', south: '↓s', east: '→e', west: '←w', up: '⬆u', down: '⬇d',
  };
  return exits.split(',').map((d) => d.trim()).map((d) => `{yellow-fg}${dirs[d] ?? d}{/}`).join('  ');
}

function wrapLines(text: string, width: number): string[] {
  const words = text.split(/\s+/);
  const lines: string[] = [];
  let line = '';
  for (const word of words) {
    if (!line) line = word;
    else if (line.length + 1 + word.length <= width) line += ` ${word}`;
    else { lines.push(line); line = word; }
  }
  if (line) lines.push(line);
  return lines;
}

function wrapBlessed(text: string, width: number): string {
  return wrapLines(text, width).join('\n');
}