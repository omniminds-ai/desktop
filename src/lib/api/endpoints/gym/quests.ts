import { apiClient } from '../../core/client';
import type { Quest } from '$lib/types/gym';
import { getReward } from '../forge/pools';

/**
 * Generate a quest based on a prompt
 * @param prompt The prompt to generate a quest from
 * @param address The user's wallet address
 * @param poolId Optional pool ID
 * @param taskId Optional task ID
 * @returns Promise resolving to the generated Quest
 */
export async function generateQuest(
  prompt: string,
  address: string,
  poolId?: string,
  taskId?: string
): Promise<Quest> {
  // Get applications from Tauri
  const apps = await import('@tauri-apps/api/core').then((module) =>
    module.invoke('list_apps', { includeIcons: true })
  );

  const app_list = (apps as { name: any; path: any }[])
    .map((app: { name: any; path: any }) => `${app.name}`)
    .join('\n');

  // Call quest endpoint
  const quest = await apiClient.post<Quest>('/gym/quest', {
    installed_applications: app_list,
    address,
    prompt
  });

  // If poolId is provided, get reward info
  if (poolId) {
    try {
      const rewardInfo = await getReward(poolId, taskId);
      quest.pool_id = poolId;
      quest.reward = { time: rewardInfo.time, max_reward: rewardInfo.maxReward };
      // If taskId is provided, add it to the quest
      if (taskId) {
        quest.task_id = taskId;
      }
    } catch (error) {
      console.error('Failed to get reward info:', error);
    }
  }

  return quest;
}

/**
 * Check the progress of a quest
 * @param quest The quest to check progress for
 * @returns Promise resolving to the progress information
 */
export async function checkQuestProgress(
  quest: Quest
): Promise<{ completed_subgoals: boolean[]; completed_objectives: number }> {
  // Get screenshots
  const screenshot = await import('@tauri-apps/api/core').then((module) =>
    module.invoke('take_screenshot')
  );

  // Call progress endpoint
  return apiClient.post<{ completed_subgoals: boolean[]; completed_objectives: number }>(
    '/gym/progress',
    {
      quest,
      screenshots: [screenshot]
    }
  );
}
