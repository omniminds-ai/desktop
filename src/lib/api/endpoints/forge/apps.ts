import { apiClient } from '../../core/client';
import type { ForgeApp } from '$lib/types/gym';
import type { TaskFilter } from '$lib/types/api';
import { invoke } from '@tauri-apps/api/core';

/**
 * Generate apps from a prompt
 * @param prompt The prompt to generate apps from
 * @returns Promise resolving to the generated content
 */
export async function generateApps(prompt: string): Promise<{
  content: {
    name: string;
    apps: ForgeApp[];
  };
}> {
  return apiClient.post<{
    content: {
      name: string;
      apps: ForgeApp[];
    };
  }>('/forge/generate', { prompt });
}

/**
 * Get apps for gym based on optional filters
 * @param filter Optional filters for apps
 * @returns Promise resolving to an array of ForgeApp objects
 */
export async function getAppsForGym(filter?: TaskFilter): Promise<ForgeApp[]> {
  const params: Record<string, any> = {};

  if (filter) {
    if (filter.poolId) params.pool_id = filter.poolId;
    if (filter.minReward) params.min_reward = filter.minReward;
    if (filter.maxReward) params.max_reward = filter.maxReward;
    if (filter.categories) params.categories = filter.categories;
    if (filter.query) params.query = filter.query;
  }

  const apps = await apiClient.get<ForgeApp[]>('/forge/apps', params);

  if (filter?.poolId) {
    return apps;
  }

  // If no poolId, shuffle apps
  const shuffledApps = apps.sort(() => Math.random() - 0.5);

  // For each app, shuffle tasks
  return shuffledApps.map((app: ForgeApp) => ({
    ...app,
    tasks: app.tasks.sort(() => Math.random() - 0.5) // Shuffle tasks
  }));
}

/**
 * Get apps for history view
 * @returns Promise resolving to an array of ForgeApp objects
 */
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

/**
 * Get apps for skills view
 * @returns Promise resolving to an array of ForgeApp objects
 */
export async function getAppsForSkills(): Promise<ForgeApp[]> {
  // Get completed apps from history
  const historyApps = await getAppsForHistory();
  const appMap = new Map<string, ForgeApp>();

  // Get submissions to get scores
  const submissions = await import('./submissions')
    .then((module) => module.listSubmissions())
    .catch(() => []);

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
    const apiApps = await apiClient.get<ForgeApp[]>('/forge/apps');
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

/**
 * Get categories for gym
 * @returns Promise resolving to an array of category strings
 */
export async function getGymCategories(): Promise<string[]> {
  return apiClient.get<string[]>('/forge/categories');
}

// generateApps function moved to generation.ts
