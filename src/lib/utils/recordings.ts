import { invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import { get } from 'svelte/store';
import { walletAddress } from '$lib/stores/wallet';
import { listSubmissions } from '$lib/api/endpoints/forge';
import type { Recording } from '$lib/types/gym';

/**
 * Deletes a recording after confirmation
 * @param recordingId ID of the recording to delete
 * @returns Promise resolving to the updated list of recordings
 */
export async function deleteRecording(recordingId: string): Promise<Recording[]> {
  const res = await ask(
    'Are you sure you want to delete this recording? This action cannot be undone.',
    {
      title: 'Delete Recording',
      kind: 'warning'
    }
  );
  if (res) {
    try {
      await invoke('delete_recording', { recordingId });
      // Refresh recordings list
      if (get(walletAddress)) await listSubmissions();
      return await invoke('list_recordings');
    } catch (error) {
      console.error('Failed to delete recording:', error);
      alert(`Error. Recording deletion failed.\n${error}`);
    }
  }
  return [];
}
