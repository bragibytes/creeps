import type { MinimapCell } from '../protocol/messages.js';
import type { World } from './world.js';

export function buildMinimap(world: World, roomId: string): MinimapCell[] {
  const zone = world.getZone(roomId);
  const cells: MinimapCell[] = [];

  for (const [id, template] of world.rooms) {
    if (template.zone !== zone || template.mapX === undefined || template.mapY === undefined) {
      continue;
    }
    cells.push({
      id,
      mapX: template.mapX,
      mapY: template.mapY,
      name: template.name.split(' ').slice(-1)[0] ?? template.name,
      current: id === roomId,
      hasExit: Object.keys(template.exits).length > 0,
    });
  }

  return cells;
}

export function renderMinimapAscii(cells: MinimapCell[]): string {
  if (cells.length === 0) return '{gray-fg}(no map){/}';

  const xs = cells.map((c) => c.mapX);
  const ys = cells.map((c) => c.mapY);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);

  const byPos = new Map<string, MinimapCell>();
  for (const c of cells) byPos.set(`${c.mapX},${c.mapY}`, c);

  const lines: string[] = [];
  for (let y = minY; y <= maxY; y++) {
    let line = '';
    for (let x = minX; x <= maxX; x++) {
      const cell = byPos.get(`${x},${y}`);
      if (!cell) {
        line += '   ';
      } else if (cell.current) {
        line += '{yellow-fg}{bold}[@]{/}';
      } else {
        line += '{gray-fg}[·]{/}';
      }
    }
    lines.push(line);
  }
  return lines.join('\n');
}