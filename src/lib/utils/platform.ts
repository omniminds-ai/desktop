import { platform } from '@tauri-apps/plugin-os';
import { dev } from '$app/environment';
import type { Platform } from '$lib/types/platform';

/**
 * Gets the current platform
 * @returns Platform identifier ('macos', 'windows', or 'linux')
 */
export async function getPlatform(): Promise<Platform> {
  const res = await platform();
  if (res === 'windows') return 'windows';
  else if (res === 'macos') return 'macos';
  else return 'linux';
}

/**
 * API URL based on environment
 */
export const API_URL = dev ? 'http://localhost' : 'https://viralmind.ai';
