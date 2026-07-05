export class DuelManager {
  private challenges = new Map<string, string>();
  private active = new Map<string, string>();

  challenge(challenger: string, target: string): string | null {
    if (challenger.toLowerCase() === target.toLowerCase()) return 'You cannot duel yourself.';
    this.challenges.set(target.toLowerCase(), challenger.toLowerCase());
    return null;
  }

  accept(accepter: string, challenger: string): string | null {
    if (this.challenges.get(accepter.toLowerCase()) !== challenger.toLowerCase()) {
      return `No duel challenge from ${challenger}.`;
    }
    this.challenges.delete(accepter.toLowerCase());
    this.active.set(accepter.toLowerCase(), challenger.toLowerCase());
    this.active.set(challenger.toLowerCase(), accepter.toLowerCase());
    return null;
  }

  isInDuel(username: string): boolean {
    return this.active.has(username.toLowerCase());
  }

  getOpponent(username: string): string | null {
    return this.active.get(username.toLowerCase()) ?? null;
  }

  endDuel(a: string, b: string): void {
    this.active.delete(a.toLowerCase());
    this.active.delete(b.toLowerCase());
  }

  onDisconnect(username: string): void {
    const opponent = this.active.get(username.toLowerCase());
    if (opponent) {
      this.active.delete(username.toLowerCase());
      this.active.delete(opponent);
    }
    this.challenges.delete(username.toLowerCase());
    for (const [target, challenger] of this.challenges) {
      if (challenger === username.toLowerCase()) this.challenges.delete(target);
    }
  }
}