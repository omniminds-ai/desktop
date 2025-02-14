import { invoke } from '@tauri-apps/api/core';

export interface MonitorInfo {
  width: number;
  height: number;
}

export interface Recording {
  id: string;
  timestamp: string;
  duration_seconds: number;
  status: string;
  title: string;
  description: string;
  platform: string;
  arch: string;
  version: string;
  locale: string;
  primary_monitor: MonitorInfo;
}

import type { Quest } from './types/gym';

export async function generateQuest(prompt: string, address: string): Promise<Quest> {
  // Get screenshot
  // const screenshot = await invoke('take_screenshot');

  // Get applications
  const apps = await invoke('list_apps', { includeIcons: true });
  const app_list = (apps as { name: any; path: any }[])
    .map((app: { name: any; path: any }) => `${app.name}`)
    .join('\n');

  // Call quest endpoint
  const response = await fetch('http://localhost/api/gym/quest', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      // screenshot,
      installed_applications: app_list,
      address,
      prompt
    })
  });

  if (!response.ok) {
    throw new Error('Failed to generate quest');
  }

  return await response.json();
}

export async function startRecording(quest?: Quest) {
  try {
    await invoke('start_recording', { quest });
  } catch (error) {
    console.error('Failed to start recording:', error);
    throw error;
  }
}

export async function stopRecording(reason?: string): Promise<string> {
  try {
    const recordingId = await invoke<string>('stop_recording', { reason });
    return recordingId;
  } catch (error) {
    console.error('Failed to stop recording:', error);
    throw error;
  }
}

export async function checkQuestProgress(
  quest: Quest
): Promise<{ completed_subgoals: boolean[]; completed_objectives: number }> {
  try {
    // Get screenshots
    const screenshots = [];
    {
      const screenshot = await invoke('take_screenshot');
      screenshots.push(screenshot);
    }

    // Call progress endpoint
    const response = await fetch('http://localhost/api/gym/progress', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        quest,
        screenshots
      })
    });

    if (!response.ok) {
      throw new Error('Failed to check progress');
    }

    return await response.json();
  } catch (error) {
    console.error('Failed to check quest progress:', error);
    throw error;
  }
}
