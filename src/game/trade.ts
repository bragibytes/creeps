export interface TradeOffer {
  items: string[];
  gold: number;
}

export interface TradeSession {
  a: string;
  b: string;
  offers: Record<string, TradeOffer>;
  confirmed: Set<string>;
}

export class TradeManager {
  private sessions = new Map<string, TradeSession>();
  private playerTrade = new Map<string, string>();
  private pending = new Map<string, string>();

  private key(a: string, b: string): string {
    return [a.toLowerCase(), b.toLowerCase()].sort().join(':');
  }

  request(from: string, to: string): string | null {
    if (from.toLowerCase() === to.toLowerCase()) return 'You cannot trade with yourself.';
    if (this.playerTrade.has(from.toLowerCase())) return 'You are already in a trade.';
    if (this.playerTrade.has(to.toLowerCase())) return 'That player is busy trading.';
    this.pending.set(to.toLowerCase(), from.toLowerCase());
    return null;
  }

  accept(accepter: string, requester: string): TradeSession | string {
    if (this.pending.get(accepter.toLowerCase()) !== requester.toLowerCase()) {
      return `No trade request from ${requester}.`;
    }
    this.pending.delete(accepter.toLowerCase());
    const sessionKey = this.key(accepter, requester);
    const session: TradeSession = {
      a: requester.toLowerCase(),
      b: accepter.toLowerCase(),
      offers: {
        [requester.toLowerCase()]: { items: [], gold: 0 },
        [accepter.toLowerCase()]: { items: [], gold: 0 },
      },
      confirmed: new Set(),
    };
    this.sessions.set(sessionKey, session);
    this.playerTrade.set(requester.toLowerCase(), sessionKey);
    this.playerTrade.set(accepter.toLowerCase(), sessionKey);
    return session;
  }

  getSession(username: string): TradeSession | null {
    const key = this.playerTrade.get(username.toLowerCase());
    return key ? this.sessions.get(key) ?? null : null;
  }

  getPartner(session: TradeSession, username: string): string {
    return session.a === username.toLowerCase() ? session.b : session.a;
  }

  offer(username: string, itemId?: string, gold?: number): string | null {
    const session = this.getSession(username);
    if (!session) return 'You are not in a trade.';
    const offer = session.offers[username.toLowerCase()];
    if (itemId) offer.items.push(itemId);
    if (gold !== undefined && gold > 0) offer.gold = gold;
    session.confirmed.clear();
    return null;
  }

  confirm(username: string): boolean {
    const session = this.getSession(username);
    if (!session) return false;
    session.confirmed.add(username.toLowerCase());
    return session.confirmed.has(session.a) && session.confirmed.has(session.b);
  }

  cancel(username: string): void {
    const key = this.playerTrade.get(username.toLowerCase());
    if (!key) return;
    const session = this.sessions.get(key);
    if (session) {
      this.playerTrade.delete(session.a);
      this.playerTrade.delete(session.b);
      this.sessions.delete(key);
    }
    for (const [to, from] of this.pending) {
      if (from === username.toLowerCase() || to === username.toLowerCase()) {
        this.pending.delete(to);
      }
    }
  }

  onDisconnect(username: string): void {
    this.cancel(username);
  }
}