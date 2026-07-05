export interface AchievementDef {
  id: string;
  name: string;
  description: string;
  title?: string;
}

export const ACHIEVEMENTS: Record<string, AchievementDef> = {
  first_blood: {
    id: 'first_blood',
    name: 'First Blood',
    description: 'Slay your first creature.',
    title: 'Initiate',
  },
  goblin_hunter: {
    id: 'goblin_hunter',
    name: 'Goblin Hunter',
    description: 'Slay 10 goblins.',
    title: 'Goblin Hunter',
  },
  chief_slayer: {
    id: 'chief_slayer',
    name: 'Kingslayer',
    description: 'Defeat Goblin Chief Grak.',
    title: 'Giantkiller',
  },
  duelist: {
    id: 'duelist',
    name: 'Duelist',
    description: 'Win your first duel.',
    title: 'Duelist',
  },
  explorer: {
    id: 'explorer',
    name: 'Explorer',
    description: 'Visit the Crystal Cavern.',
    title: 'Pathfinder',
  },
  wealthy: {
    id: 'wealthy',
    name: 'Wealthy',
    description: 'Accumulate 500 gold.',
    title: 'Magnate',
  },
  veteran: {
    id: 'veteran',
    name: 'Veteran',
    description: 'Reach level 10.',
    title: 'Veteran',
  },
};

export function grantAchievement(
  earned: string[],
  id: string,
): AchievementDef | null {
  if (earned.includes(id)) return null;
  earned.push(id);
  return ACHIEVEMENTS[id] ?? null;
}