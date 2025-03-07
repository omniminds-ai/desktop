import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { dev } from '$app/environment';
import { platform } from '@tauri-apps/plugin-os';

export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

export async function getPlatform(): Promise<'macos' | 'windows' | 'linux'> {
  const res = await platform();
  if (res === 'windows') return 'windows';
  else if (res === 'macos') return 'macos';
  else return 'linux';
}

export function svgToCssTransform(svgTransform: string): string {
  return svgTransform
    .replace(/translate\(([-\d.,]+),([-\d.,]+)\)/, (_, x, y) => `translate(${x}px, ${y}px)`)
    .replace(/scale\(([-\d.,]+)\)/, (_, s) => `scale(${s})`);
}

export const API_URL = dev ? 'http://localhost' : 'https://viralmind.ai';

export const checkForUpdate = async (): Promise<null | Update> => {
  const updater = await check();
  if (updater) {
    console.log(
      `[Tauri Updater] Found update ${updater.version} from ${updater.date} with notes ${updater.body}`
    );
  }
  return updater;
};

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
