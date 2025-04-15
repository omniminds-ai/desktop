import { apiClient } from '$lib/api/core';

/**
 * Generate apps from a prompt
 * @param prompt The prompt to generate apps from
 * @returns Promise resolving to the generated content
 */
export async function checkConnection(token: string): Promise<{
  connected: boolean;
  address?: string;
}> {
  return apiClient.get<{
    connected: boolean;
    address?: string;
  }>('/wallet/connection', { token });
}

/**
 * Generate apps from a prompt
 * @param prompt The prompt to generate apps from
 * @returns Promise resolving to the generated content
 */
export async function getNickname(address: string): Promise<string> {
  return apiClient.get<string>('/wallet/nickname', { address });
}
export async function setNickname(address: string, nickname: string): Promise<string> {
  return apiClient.put<string>('/wallet/nickname', { address, nickname }, { requiresAuth: true });
}
