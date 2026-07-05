import type { World } from './world.js';
import { nextMobId } from './world.js';

export type GlobalNotifyFn = (text: string) => void;

export class WorldEventManager {
  private raidActive = false;
  private interval: ReturnType<typeof setInterval>;

  constructor(
    private world: World,
    private notify: GlobalNotifyFn,
  ) {
    this.interval = setInterval(() => this.tick(), 5 * 60_000);
  }

  private tick(): void {
    if (this.raidActive) return;
    if (Math.random() > 0.35) return;
    this.startGoblinRaid();
  }

  startGoblinRaid(): void {
    const room = this.world.getRoom('eldermoor_square');
    if (!room) return;

    this.raidActive = true;
    this.notify('*** WORLD EVENT: Goblins raid Eldermoor Town Square! ***');

    for (let i = 0; i < 3; i++) {
      const tmpl = this.world.mobs.get('goblin_scout');
      if (tmpl) {
        room.mobs.push({
          instanceId: nextMobId(),
          templateId: 'goblin_scout',
          hp: tmpl.hp,
          maxHp: tmpl.hp,
        });
      }
    }

    setTimeout(() => {
      this.raidActive = false;
      this.notify('The goblin raid on Eldermoor has ended.');
    }, 5 * 60_000);
  }

  stop(): void {
    clearInterval(this.interval);
  }
}