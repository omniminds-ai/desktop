import { walletAddress } from '$lib/stores/wallet';
import type {
  TrainingPool,
  CreatePoolInput,
  UpdatePoolInput,
  SubmissionStatus
} from '$lib/types/forge';
import type { ForgeApp } from '$lib/types/gym';
import { API_URL } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';

const API_BASE = `${API_URL}/api/forge`;

export async function getAppsForGym(poolId?: string): Promise<ForgeApp[]> {
  const response = await fetch(`${API_BASE}/apps${poolId ? `?pool_id=${poolId}` : ''}`);

  if (!response.ok) {
    throw new Error('Failed to fetch apps');
  }

  const apps: ForgeApp[] = await response.json();

  if (poolId) {
    return apps;
  }

  // If no poolId, shuffle and limit apps
  const shuffledApps = apps.sort(() => Math.random() - 0.5).slice(0, 6);

  // For each app, keep only a random subset of tasks
  return shuffledApps.map((app: ForgeApp) => ({
    ...app,
    tasks: app.tasks
      .sort(() => Math.random() - 0.5) // Shuffle tasks
      .slice(0, 3) // Keep max 3 tasks per app
  }));
}

export async function getAppsForHistory(): Promise<ForgeApp[]> {
  // Get recordings to build app map
  const recordings: any[] = await invoke('list_recordings');
  const appMap = new Map<string, ForgeApp>();

  // Build apps from recordings
  recordings.forEach((recording) => {
    if (recording.status === 'completed' && recording.quest) {
      const quest = recording.quest;
      if (!appMap.has(quest.app)) {
        appMap.set(quest.app, {
          name: quest.app,
          domain: quest?.icon_url?.split('domain=')[1]?.split('&')[0],
          description: '',
          categories: [],
          tasks: [],
          pool_id: { _id: '', name: '', status: 'completed', pricePerDemo: 0 },
          seen: true
        });
      }
      const app = appMap.get(quest.app)!;
      if (!app.tasks.some((t) => t.prompt === quest.title)) {
        app.tasks.push({
          prompt: quest.title,
          completed: true,
          recordingId: recording.id
        });
      }
    }
  });

  // Convert map to array and sort by app name
  return Array.from(appMap.values()).sort((a, b) => a.name.localeCompare(b.name));
}

export async function getAppsForSkills(): Promise<ForgeApp[]> {
  // Get completed apps from history
  const historyApps = await getAppsForHistory();
  const appMap = new Map<string, ForgeApp>();

  // Get submissions to get scores
  const address = get(walletAddress);
  let submissions: SubmissionStatus[] = [];
  if (address) {
    submissions = await listSubmissions(address);
  }

  // Add history apps to map
  historyApps.forEach((app) => {
    // Update tasks with submission scores
    app.tasks = app.tasks.map((task) => {
      if (task.recordingId) {
        const submission = submissions.find((s) => s.meta?.id === task.recordingId);
        if (submission?.clampedScore) {
          return { ...task, score: submission.clampedScore };
        }
      }
      return task;
    });
    appMap.set(app.name, app);
  });

  // Then get apps from API and merge
  try {
    const response = await fetch(`${API_BASE}/apps`);
    if (response.ok) {
      const apiApps: ForgeApp[] = await response.json();
      apiApps.forEach((apiApp) => {
        const existingApp = appMap.get(apiApp.name);
        if (existingApp) {
          // Merge API app info into existing app
          existingApp.domain = apiApp.domain;
          existingApp.description = apiApp.description;
          existingApp.categories = apiApp.categories;
          // Add any new tasks from API
          apiApp.tasks.forEach((apiTask) => {
            if (!existingApp.tasks.some((t) => t.prompt === apiTask.prompt)) {
              existingApp.tasks.push({
                ...apiTask,
                completed: false
              });
            }
          });
        } else {
          // Add new app from API
          appMap.set(apiApp.name, {
            ...apiApp,
            seen: false,
            tasks: apiApp.tasks.map((task) => ({
              ...task,
              completed: false
            }))
          });
        }
      });
    }
  } catch (error) {
    console.error('Failed to fetch apps from API:', error);
  }

  // Convert map to array
  let allApps = Array.from(appMap.values());

  // If we have no apps at all, return empty array
  if (allApps.length === 0) {
    return [];
  }

  // Split into seen and unseen
  const seen = allApps.filter((app) => app.seen);
  const unseen = allApps.filter((app) => !app.seen);

  // Always include all seen apps
  let result = [...seen];

  // Add random unseen apps if we have any
  if (unseen.length > 0) {
    const shuffledUnseen = unseen.sort(() => Math.random() - 0.5);
    // Add up to 2 unseen apps, but don't exceed total of 3 apps
    const unseenToAdd = Math.min(2, Math.max(0, 3 - seen.length));
    result = [...result, ...shuffledUnseen.slice(0, unseenToAdd)];
  }

  return result;
}

export async function listPools(address: string): Promise<TrainingPool[]> {
  const response = await fetch(`${API_BASE}/list`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ address })
  });

  if (!response.ok) {
    throw new Error('Failed to fetch training pools');
  }

  return response.json();
}

export async function createPool(input: CreatePoolInput): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(input)
  });

  if (!response.ok) {
    throw new Error('Failed to create training pool');
  }

  return response.json();
}

export async function updatePool(input: UpdatePoolInput): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/update`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(input)
  });

  if (!response.ok) {
    throw new Error('Failed to update training pool');
  }

  return response.json();
}

export interface RewardInfo {
  time: number; // Unix timestamp rounded to last minute
  maxReward: number; // Value between 1-128
}

export async function getBalance(address: string): Promise<number> {
  const response = await fetch(`${API_BASE}/balance/${address}`);

  if (!response.ok) {
    throw new Error('Failed to get balance');
  }

  const data = await response.json();
  return data.balance;
}

export async function getReward(poolId: string, address: string): Promise<RewardInfo> {
  const response = await fetch(`${API_BASE}/reward?poolId=${poolId}&address=${address}`);

  if (!response.ok) {
    throw new Error('Failed to get reward info');
  }

  return response.json();
}

export async function uploadRecording(
  zipBlob: Blob,
  address: string,
  onProgress?: (progress: number) => void
): Promise<{ submissionId: string }> {
  const formData = new FormData();
  formData.append('file', zipBlob, 'recording.zip');

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();

    // Track upload progress
    xhr.upload.addEventListener('progress', (event) => {
      if (event.lengthComputable && onProgress) {
        // Calculate percentage (0-100)
        const progress = Math.round((event.loaded / event.total) * 100);
        onProgress(progress);
      }
    });

    xhr.addEventListener('load', () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          const response = JSON.parse(xhr.responseText);
          resolve(response);
        } catch (error) {
          reject(new Error('Invalid response format'));
        }
      } else {
        reject(new Error(`Failed to upload recording: ${xhr.status} ${xhr.statusText}`));
      }
    });

    xhr.addEventListener('error', () => {
      reject(new Error('Network error occurred during upload'));
    });

    xhr.addEventListener('abort', () => {
      reject(new Error('Upload was aborted'));
    });

    xhr.open('POST', `${API_BASE}/upload-race`);
    xhr.setRequestHeader('x-wallet-address', address);
    xhr.send(formData);
  });
}

export async function getSubmissionStatus(submissionId: string): Promise<SubmissionStatus> {
  const response = await fetch(`${API_BASE}/submission/${submissionId}`);

  if (!response.ok) {
    throw new Error('Failed to get submission status');
  }

  return response.json();
}

export async function listSubmissions(address: string): Promise<SubmissionStatus[]> {
  const response = await fetch(`${API_BASE}/submissions`, {
    headers: {
      'x-wallet-address': address
    }
  });

  if (!response.ok) {
    throw new Error('Failed to list submissions');
  }

  return response.json();
}

export async function refreshPool(poolId: string): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/refresh`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ id: poolId })
  });

  if (!response.ok) {
    throw new Error('Failed to refresh pool');
  }

  return response.json();
}

export interface GenerateResponse {
  content: {
    name: string;
    apps: ForgeApp[];
  }
}

export async function generateApps(prompt: string): Promise<GenerateResponse> {
  const response = await fetch(`${API_BASE}/generate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ prompt })
  });

  if (!response.ok) {
    throw new Error('Failed to generate apps from prompt');
  }

  return await response.json();
}

export interface PoolSubmission {
  _id: string;
  address: string;
  meta: {
    poolId: string;
    id: string;
    generatedTime: number;
    quest: {
      pool_id: string;
      app_id: string;
      title?: string;
      app?: string;
    };
    [key: string]: any;
  };
  status: string; // PENDING, PROCESSING, COMPLETED, FAILED, etc.
  files: {
    file: string;
    s3Key: string;
    _id?: string;
    size?: number;
  }[];
  grade_result?: any;
  reward: number;
  maxReward: number;
  clampedScore: number;
  createdAt: string;
  updatedAt: string;
}

export async function getPoolSubmissions(poolId: string): Promise<PoolSubmission[]> {
  const response = await fetch(`${API_BASE}/pool-submissions/${poolId}`);

  if (!response.ok) {
    throw new Error('Failed to fetch pool submissions');
  }

  return response.json();
}

// Update create pool interface to accept apps
export interface CreatePoolInputWithApps extends CreatePoolInput {
  apps?: ForgeApp[];
}

// Updated create pool function
export async function createPoolWithApps(input: CreatePoolInputWithApps): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(input)
  });

  if (!response.ok) {
    throw new Error('Failed to create training pool');
  }

  return response.json();
}
