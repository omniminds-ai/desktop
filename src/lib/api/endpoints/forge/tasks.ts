import { apiClient } from '../../core/client';
import type { ForgeTask } from '$lib/types/gym';
import type { TaskFilter } from '$lib/types/api';

/**
 * Get tasks for gym based on optional filters
 * @param filter Optional filters for tasks
 * @returns Promise resolving to an array of ForgeTask objects
 */
export async function getTasksForGym(filter?: TaskFilter): Promise<ForgeTask[]> {
  // Convert filter parameters to API-friendly format if needed
  const params: Record<string, any> = {};

  if (filter) {
    if (filter.poolId) params.pool_id = filter.poolId;
    if (filter.minReward) params.min_reward = filter.minReward;
    if (filter.maxReward) params.max_reward = filter.maxReward;
    if (filter.categories) params.categories = filter.categories;
    if (filter.query) params.query = filter.query;
    if (filter.hideAdult !== undefined) params.hide_adult = filter.hideAdult;
  }

  // Use the API client to make the request
  const tasks = await apiClient.get<ForgeTask[]>('/forge/apps/tasks', params);

  return tasks;
}
