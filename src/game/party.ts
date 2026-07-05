import type { PlayerSession } from './player.js';

export class PartyManager {
  private leaderOf = new Map<string, Set<string>>();
  private memberOf = new Map<string, string>();
  private invites = new Map<string, string>();

  invite(leader: PlayerSession, target: PlayerSession): string | null {
    if (this.memberOf.has(target.username.toLowerCase())) {
      return `${target.username} is already in a party.`;
    }
    const leaderKey = leader.username.toLowerCase();
    const memberLeader = this.memberOf.get(leaderKey);
    if (memberLeader && memberLeader !== leaderKey) {
      return 'Only the party leader can invite.';
    }
    this.ensureParty(leader.username);
    this.invites.set(target.username.toLowerCase(), leaderKey);
    return null;
  }

  join(player: PlayerSession, leaderName: string): string | null {
    const leaderKey = leaderName.toLowerCase();
    if (this.invites.get(player.username.toLowerCase()) !== leaderKey) {
      return `No party invite from ${leaderName}.`;
    }
    this.invites.delete(player.username.toLowerCase());
    this.ensureParty(leaderName);
    this.leaderOf.get(leaderKey)!.add(player.username.toLowerCase());
    this.memberOf.set(player.username.toLowerCase(), leaderKey);
    return null;
  }

  leave(player: PlayerSession): void {
    const leaderKey = this.memberOf.get(player.username.toLowerCase());
    if (!leaderKey) return;
    const party = this.leaderOf.get(leaderKey);
    party?.delete(player.username.toLowerCase());
    this.memberOf.delete(player.username.toLowerCase());
    if (leaderKey === player.username.toLowerCase()) {
      const members = [...(party ?? [])];
      if (members.length > 0) {
        const newLeader = members[0];
        const rest = new Set(members.slice(1));
        this.leaderOf.delete(leaderKey);
        this.leaderOf.set(newLeader, rest);
        for (const m of members) this.memberOf.set(m, newLeader);
      } else {
        this.leaderOf.delete(leaderKey);
      }
    }
  }

  getLeaderKey(player: PlayerSession): string | null {
    return this.memberOf.get(player.username.toLowerCase()) ?? null;
  }

  getMemberUsernames(leaderKey: string, players: Map<string, PlayerSession>): string[] {
    const party = this.leaderOf.get(leaderKey);
    const leader = players.get(leaderKey);
    if (!party || !leader) return leader ? [leader.username] : [];
    const names = [leader.username];
    for (const m of party) {
      const p = players.get(m);
      if (p) names.push(p.username);
    }
    return names;
  }

  getPartyPeers(player: PlayerSession, players: Map<string, PlayerSession>): PlayerSession[] {
    const leaderKey = this.memberOf.get(player.username.toLowerCase()) ?? player.username.toLowerCase();
    return this.getMemberUsernames(leaderKey, players)
      .map((n) => players.get(n.toLowerCase()))
      .filter((p): p is PlayerSession => !!p && p.authenticated);
  }

  private ensureParty(leaderUsername: string): void {
    const key = leaderUsername.toLowerCase();
    if (!this.leaderOf.has(key)) {
      this.leaderOf.set(key, new Set());
      this.memberOf.set(key, key);
    }
  }

  onDisconnect(player: PlayerSession): void {
    this.leave(player);
    this.invites.delete(player.username.toLowerCase());
    for (const [target, leader] of this.invites) {
      if (leader === player.username.toLowerCase()) this.invites.delete(target);
    }
  }
}