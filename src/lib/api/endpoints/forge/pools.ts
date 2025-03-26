import { apiClient } from '../../core/client';
import type { TrainingPool, CreatePoolInput, UpdatePoolInput } from '$lib/types/forge';
import type { ForgeApp } from '$lib/types/gym';

/**
 * List all training pools
 * @returns Promise resolving to an array of TrainingPool objects
 */
export async function listPools(): Promise<TrainingPool[]> {
  return apiClient.post<TrainingPool[]>('/forge/list', {}, { requiresAuth: true });
}

/**
 * Create a new training pool
 * @param input The pool creation input
 * @returns Promise resolving to the created TrainingPool
 */
export async function createPool(input: CreatePoolInput): Promise<TrainingPool> {
  return apiClient.post<TrainingPool>('/forge/create', input, { requiresAuth: true });
}

/**
 * Update an existing training pool
 * @param input The pool update input
 * @returns Promise resolving to the updated TrainingPool
 */
export async function updatePool(input: UpdatePoolInput): Promise<TrainingPool> {
  return apiClient.post<TrainingPool>('/forge/update', input, { requiresAuth: true });
}

/**
 * Refresh a training pool
 * @param poolId The ID of the pool to refresh
 * @returns Promise resolving to the refreshed TrainingPool
 */
export async function refreshPool(poolId: string): Promise<TrainingPool> {
  return apiClient.post<TrainingPool>('/forge/refresh', { id: poolId }, { requiresAuth: true });
}

/**
 * Get the user's balance
 * @param address The wallet address
 * @returns Promise resolving to the balance
 */
export async function getBalance(address: string): Promise<number> {
  const data = await apiClient.get<{ balance: number }>(`/forge/balance/${address}`, {}, { requiresAuth: true });
  return data.balance;
}

/**
 * Get reward information for a pool
 * @param poolId The ID of the pool
 * @param taskId Optional task ID
 * @returns Promise resolving to reward information
 */
export async function getReward(poolId: string, taskId?: string): Promise<{
  time: number;
  maxReward: number;
}> {
  const params: Record<string, any> = { poolId };
  if (taskId) {
    params.taskId = taskId;
  }
  
  return apiClient.get<{ time: number; maxReward: number }>('/forge/reward', params, { requiresAuth: true });
}

/**
 * Create a pool with apps
 * @param input The pool creation input with apps
 * @returns Promise resolving to the created TrainingPool
 */
export async function createPoolWithApps(input: CreatePoolInput & { apps?: ForgeApp[] }): Promise<TrainingPool> {
  return apiClient.post<TrainingPool>('/forge/create', input, { requiresAuth: true });
}
