import { writable } from 'svelte/store';
import type { ToolsInitState } from '$lib/types/tools';

/**
 * Store for tracking tools initialization state
 */
export const toolsInitState = writable<ToolsInitState>({
  initializing: false,
  progress: 5
});
