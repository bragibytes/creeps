#!/usr/bin/env node
import WebSocket from 'ws';
import type { ClientMessage, ServerMessage } from '../protocol/messages.js';
import { HOTKEY_COMMANDS } from '../game/types.js';
import { createPlainClient } from './plain.js';
import { createTuiClient } from './tui.js';
import type { GameClientUI } from './plain.js';

const SERVER_URL = process.env.REALM_SERVER ?? 'ws://localhost:4242';
const USE_PLAIN = process.argv.includes('--plain') || process.env.REALM_PLAIN === '1' || !process.stdout.isTTY;
const MAX_RECONNECT = 5;

let ws: WebSocket;
let ui: GameClientUI;
let authenticated = false;
let authStep: 'mode' | 'username' | 'password' | 'class' = 'mode';
let pendingUsername = '';
let pendingPassword = '';
let savedUsername = '';
let savedPassword = '';
let reconnectAttempts = 0;
let intentionalDisconnect = false;

function main(): void {
  ui = USE_PLAIN ? createPlainClient() : createTuiClient();
  ui.onInput(onInput);
  connect();
}

function connect(): void {
  ws = new WebSocket(SERVER_URL);

  ws.on('open', () => {
    reconnectAttempts = 0;
    if (authenticated && savedUsername && savedPassword) {
      send({ type: 'login', username: savedUsername, password: savedPassword });
    }
  });

  ws.on('message', (data) => {
    handleMessage(JSON.parse(data.toString()) as ServerMessage);
  });

  ws.on('close', () => {
    if (intentionalDisconnect) {
      ui.showDisconnect('Farewell, adventurer!');
      ui.destroy();
      process.exit(0);
      return;
    }
    if (authenticated && reconnectAttempts < MAX_RECONNECT) {
      reconnectAttempts++;
      ui.log(`Connection lost. Reconnecting (${reconnectAttempts}/${MAX_RECONNECT})...`, 'system');
      setTimeout(connect, 2000);
      return;
    }
    ui.showDisconnect('Disconnected from server.');
    ui.destroy();
    process.exit(0);
  });

  ws.on('error', (err) => {
    if (reconnectAttempts === 0 && !authenticated) {
      ui.showError(`Connection error: ${err.message}`);
      ui.log('Is the server running? Start it with: npm run server', 'system');
      ui.destroy();
      process.exit(1);
    }
  });
}

function send(msg: ClientMessage): void {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(msg));
  }
}

function expandInput(raw: string): string {
  const trimmed = raw.trim();
  const lower = trimmed.toLowerCase();
  if (!lower.includes(' ') && HOTKEY_COMMANDS[lower]) {
    return HOTKEY_COMMANDS[lower];
  }
  return trimmed;
}

function handleMessage(msg: ServerMessage): void {
  switch (msg.type) {
    case 'banner':
      ui.showBanner();
      break;

    case 'output':
      ui.log(msg.text, msg.style ?? 'normal');
      break;

    case 'room':
      ui.showRoom({
        title: msg.title,
        description: msg.description,
        exits: msg.exits,
        entities: msg.entities,
        zone: msg.zone,
        minimap: msg.minimap,
        zoneArt: msg.zoneArt,
      });
      break;

    case 'stats':
      ui.showStats(msg.player);
      break;

    case 'online':
      ui.showOnline(msg.players);
      break;

    case 'flash':
      ui.flash(msg.color);
      break;

    case 'error':
      ui.showError(msg.text);
      break;

    case 'prompt':
      ui.setPrompt(authenticated ? '>' : msg.text.trim() || 'login or register?');
      break;

    case 'disconnect':
      intentionalDisconnect = true;
      ui.showDisconnect(msg.reason);
      ws.close();
      break;
  }
}

function onInput(line: string): void {
  const input = expandInput(line);
  if (!input) {
    if (authenticated) ui.setPrompt('>');
    return;
  }

  if (input.toLowerCase() === 'quit' || input.toLowerCase() === 'exit') {
    if (authenticated) {
      intentionalDisconnect = true;
      send({ type: 'command', input });
    }
    return;
  }

  if (!authenticated) {
    handleAuth(input);
    return;
  }

  send({ type: 'command', input });
}

function handleAuth(input: string): void {
  const lower = input.toLowerCase();

  switch (authStep) {
    case 'mode':
      if (lower === 'login') {
        authStep = 'username';
        ui.setPrompt('username:');
        ui.log('Enter your username.', 'system');
      } else if (lower === 'register') {
        authStep = 'username';
        pendingPassword = '__register__';
        ui.setPrompt('username:');
        ui.log('Choose a username (3-16 chars).', 'system');
      } else {
        ui.showError('Type "login" or "register".');
        ui.setPrompt('login or register?');
      }
      break;

    case 'username':
      pendingUsername = input;
      authStep = 'password';
      ui.setPrompt('password:', true);
      ui.log('Enter password.', 'system');
      break;

    case 'password':
      if (pendingPassword === '__register__') {
        pendingPassword = input;
        authStep = 'class';
        ui.setPrompt('class:');
        ui.showClassSelect();
      } else {
        savedUsername = pendingUsername;
        savedPassword = input;
        send({ type: 'login', username: pendingUsername, password: input });
        authenticated = true;
        authStep = 'mode';
        ui.setPrompt('>');
      }
      break;

    case 'class': {
      const className = lower as 'warrior' | 'mage' | 'rogue';
      if (!['warrior', 'mage', 'rogue'].includes(className)) {
        ui.showError('Choose: warrior, mage, or rogue');
        ui.setPrompt('class:');
        return;
      }
      savedUsername = pendingUsername;
      savedPassword = pendingPassword;
      send({ type: 'register', username: pendingUsername, password: pendingPassword, className });
      authenticated = true;
      authStep = 'mode';
      ui.setPrompt('>');
      break;
    }
  }
}

main();