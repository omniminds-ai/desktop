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
