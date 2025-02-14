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
