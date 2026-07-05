import { copyFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from 'fs';
import { join } from 'path';

const MAX_BACKUPS = 5;

export function backupPlayersJson(dataDir: string, storePath: string): void {
  if (!existsSync(storePath)) return;
  const backupDir = join(dataDir, 'backups');
  mkdirSync(backupDir, { recursive: true });
  const stamp = new Date().toISOString().replace(/[:.]/g, '-');
  const dest = join(backupDir, `players-${stamp}.json`);
  copyFileSync(storePath, dest);

  const files = readdirSync(backupDir)
    .filter((f) => f.startsWith('players-') && f.endsWith('.json'))
    .sort()
    .reverse();
  for (const old of files.slice(MAX_BACKUPS)) {
    unlinkSync(join(backupDir, old));
  }
}