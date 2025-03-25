import { invoke } from '@tauri-apps/api/core';
import { getReward } from './forge';

import type { Quest } from '../types/gym';
import { API_URL } from '../utils';

export async function generateQuest(
  prompt: string,
  address: string,
  poolId?: string,
  taskId?: string
): Promise<Quest> {
  // Get screenshot
  // const screenshot = await invoke('take_screenshot');

  // Get applications
  const apps = await invoke('list_apps', { includeIcons: true });
  const app_list = (apps as { name: any; path: any }[])
    .map((app: { name: any; path: any }) => `${app.name}`)
    .join('\n');

  // Call quest endpoint
  const response = await fetch(`${API_URL}/api/gym/quest`, {
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

  const quest = await response.json();

  // If poolId is provided, get reward info
  if (poolId) {
    try {
      const rewardInfo = await getReward(poolId, taskId);
      quest.poolId = poolId;
      quest.reward = rewardInfo;
      // If taskId is provided, add it to the quest
      if (taskId) {
        quest.task_id = taskId;
      }
    } catch (error) {
      console.error('Failed to get reward info:', error);
    }
  }

  return quest;
}

export async function startRecording(quest?: Quest) {
  try {
    // If quest has reward info, include poolId and generatedTime in meta
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
    const response = await fetch(`${API_URL}/api/gym/progress`, {
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
