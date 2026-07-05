import { GameServer } from './game-server.js';

const PORT = parseInt(process.env.PORT ?? '4242', 10);
const server = new GameServer(PORT);

function shutdown(): void {
  console.log('\nShutting down...');
  server.shutdown();
  process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);