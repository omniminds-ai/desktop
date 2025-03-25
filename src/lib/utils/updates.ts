import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { writable } from 'svelte/store';
import type { ToolsInitState } from '$lib/types/updates';

/**
 * Store for tracking tools initialization state
 */
export const toolsInitState = writable<ToolsInitState>({
  initializing: false,
  progress: 5
});

/**
 * Checks for application updates
 * @returns Promise resolving to an Update object if an update is available, or null if no update is available
 */
export const checkForUpdate = async (): Promise<null | Update> => {
  const updater = await check();
  if (updater) {
    console.log(
      `[Tauri Updater] Found update ${updater.version} from ${updater.date} with notes ${updater.body}`
    );
  }
  return updater;
};

/**
 * Downloads and installs an application update
 * @param updater The Update object to install
 */
export const updateApp = async (updater: Update) => {
  let downloaded = 0;
  let contentLength = 0;
  // alternatively we could also call update.download() and update.install() separately
  await updater.downloadAndInstall((event) => {
    switch (event.event) {
      case 'Started':
        contentLength = event.data.contentLength || 0;
        console.log(`[Tauri Updater] Started downloading ${event.data.contentLength} bytes`);
        break;
      case 'Progress':
        downloaded += event.data.chunkLength;
        console.log(`[Tauri Updater] Downloaded ${downloaded} from ${contentLength}`);
        break;
      case 'Finished':
        console.log('[Tauri Updater] Download finished');
        break;
    }
  });

  console.log('[Tauri Updater] Update installed');
  await relaunch();
};
