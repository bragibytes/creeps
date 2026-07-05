import type { QuestTemplate } from './types.js';

export interface QuestProgress {
  questId: string;
  status: 'active' | 'complete' | 'turned_in';
  progress: Record<string, number>;
}

export function createQuestProgress(quest: QuestTemplate): QuestProgress {
  const progress: Record<string, number> = {};
  for (const obj of quest.objectives) {
    progress[obj.target] = 0;
  }
  return { questId: quest.id, status: 'active', progress };
}

export function checkQuestComplete(quest: QuestTemplate, qp: QuestProgress): boolean {
  return quest.objectives.every((obj) => (qp.progress[obj.target] ?? 0) >= obj.count);
}

export function formatQuestProgress(quest: QuestTemplate, qp: QuestProgress): string {
  const lines = [`[${quest.name}] ${quest.description}`, 'Objectives:'];
  for (const obj of quest.objectives) {
    const current = qp.progress[obj.target] ?? 0;
    const done = current >= obj.count;
    lines.push(`  ${done ? '✓' : '○'} ${obj.description} (${current}/${obj.count})`);
  }
  return lines.join('\n');
}