import type {Quest, Screen} from '$lib/types/gym';
import { invoke } from '@tauri-apps/api/core';

/**
 * Start recording a quest
 * @param quest Optional quest to record
 * @returns Promise that resolves when recording starts
 */
export async function startRecording(quest?: Quest, screen?: Screen): Promise<void> {
  try {
    // If quest has reward info, include poolId and generatedTime in meta
    console.log({quest, screen})

    await invoke('start_recording', {quest, display: screen});
  } catch (error) {
    console.error('Failed to start recording:', error);
    throw error;
  }
}

/**
 * Stop recording
 * @param reason Optional reason for stopping
 * @returns Promise resolving to the recording ID
 */
export async function stopRecording(reason?: string): Promise<string> {
  try {
    const recordingId = await invoke<string>('stop_recording', { reason });
    return recordingId;
  } catch (error) {
    console.error('Failed to stop recording:', error);
    throw error;
  }
}
