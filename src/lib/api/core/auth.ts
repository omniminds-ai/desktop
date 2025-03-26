import { get } from 'svelte/store';
import { connectionToken, walletAddress } from '$lib/stores/wallet';

/**
 * Authentication utilities for API requests
 */

/**
 * Gets the current authentication token
 * @returns The current connection token or empty string
 */
export function getAuthToken(): string {
  return get(connectionToken) || '';
}

/**
 * Gets the current wallet address
 * @returns The current wallet address or null
 */
export function getCurrentWalletAddress(): string | null {
  return get(walletAddress);
}

/**
 * Checks if the user is authenticated
 * @returns True if the user has a connection token
 */
export function isAuthenticated(): boolean {
  return !!getAuthToken();
}

/**
 * Creates authentication headers for API requests
 * @returns Object with authentication headers
 */
export function getAuthHeaders(): Record<string, string> {
  const token = getAuthToken();
  return token ? { 'x-connect-token': token } : {};
}
