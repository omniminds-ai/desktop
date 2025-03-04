import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { walletAddress } from '$lib/stores/wallet';
import { uploadRecording, getSubmissionStatus } from '$lib/api/forge';

// Store for tracking if upload confirmation is needed
export const needsUploadConfirmation = writable<boolean>(true);

// Function to check if uploads have been confirmed
export async function checkUploadConfirmed(): Promise<boolean> {
  try {
    const confirmed = await invoke<boolean>('get_upload_confirmed');
    needsUploadConfirmation.set(!confirmed);
    return confirmed;
  } catch (error) {
    console.error('Failed to check upload confirmation status:', error);
    return false;
  }
}

// Function to save upload confirmation
export async function saveUploadConfirmation(confirmed: boolean): Promise<void> {
  try {
    await invoke('set_upload_confirmed', { confirmed });
    needsUploadConfirmation.set(!confirmed);
  } catch (error) {
    console.error('Failed to save upload confirmation:', error);
  }
}

// Store for tracking uploads
export const uploadQueue = writable<{
  [recordingId: string]: {
    status: 'queued' | 'uploading' | 'zipping' | 'processing' | 'completed' | 'failed';
    progress?: number;
    error?: string;
    submissionId?: string;
    name?: string;
  };
}>({});

// Derived store to check if any uploads are active
export const hasActiveUploads = derived(uploadQueue, ($queue) => {
  return Object.values($queue).some(
    (item) =>
      item.status === 'queued' || item.status === 'uploading' || item.status === 'processing'
  );
});

// Status intervals for polling
const statusIntervals: { [recordingId: string]: number } = {};

// Function to clean up intervals
export function cleanupIntervals() {
  Object.values(statusIntervals).forEach((interval) => {
    clearInterval(interval);
  });
}

// Function to poll submission status
function pollSubmissionStatus(recordingId: string, submissionId: string) {
  if (statusIntervals[recordingId]) {
    clearInterval(statusIntervals[recordingId]);
  }

  statusIntervals[recordingId] = setInterval(async () => {
    try {
      const status = await getSubmissionStatus(submissionId);

      if (status.status === 'completed') {
        uploadQueue.update((queue) => {
          queue[recordingId] = {
            ...queue[recordingId],
            status: 'completed',
            progress: 100
          };
          return queue;
        });
        clearInterval(statusIntervals[recordingId]);
        delete statusIntervals[recordingId];

        // Auto-remove completed uploads after 5 seconds
        setTimeout(() => {
          uploadQueue.update((queue) => {
            const newQueue = { ...queue };
            delete newQueue[recordingId];
            return newQueue;
          });
        }, 5000);
      } else if (status.status === 'failed') {
        uploadQueue.update((queue) => {
          queue[recordingId] = {
            ...queue[recordingId],
            status: 'failed',
            error: status.error || 'Upload failed'
          };
          return queue;
        });
        clearInterval(statusIntervals[recordingId]);
        delete statusIntervals[recordingId];
      } else if (status.status === 'processing') {
        uploadQueue.update((queue) => {
          queue[recordingId] = {
            ...queue[recordingId],
            status: 'processing',
            progress: 80 // Arbitrary progress for processing state
          };
          return queue;
        });
      }
    } catch (error) {
      console.error('Failed to get submission status:', error);
      uploadQueue.update((queue) => {
        queue[recordingId] = {
          ...queue[recordingId],
          status: 'failed',
          error: error instanceof Error ? error.message : 'Failed to check status'
        };
        return queue;
      });
      clearInterval(statusIntervals[recordingId]);
      delete statusIntervals[recordingId];
    }
  }, 5000);
}

// Main upload function that will be called from other components
export async function handleUpload(recordingId: string, name: string): Promise<boolean> {
  if (!get(walletAddress)) {
    uploadQueue.update((queue) => {
      queue[recordingId] = {
        status: 'failed',
        error: 'Please connect your wallet first'
      };
      return queue;
    });
    return false;
  }

  // Check if upload has been confirmed
  const isConfirmed = await checkUploadConfirmed();
  if (!isConfirmed) {
    // Return false to indicate that confirmation is needed
    // The calling component should show the confirmation modal
    return false;
  }

  // If we get here, either the user has already confirmed uploads or they just confirmed
  // Add to queue
  uploadQueue.update((queue) => {
    queue[recordingId] = { status: 'queued', name };
    return queue;
  });

  try {
    // Update status to uploading
    uploadQueue.update((queue) => {
      queue[recordingId] = { status: 'zipping', name, progress: 0 };
      return queue;
    });

    // Get zip file as bytes
    const zipBytes = await invoke<number[]>('create_recording_zip', { recordingId });

    // Update progress
    uploadQueue.update((queue) => {
      queue[recordingId] = { status: 'uploading', name, progress: 30 };
      return queue;
    });

    // Convert to Blob
    const zipBlob = new Blob([Uint8Array.from(zipBytes)], { type: 'application/zip' });

    // Update progress
    uploadQueue.update((queue) => {
      queue[recordingId] = { status: 'uploading', name, progress: 60 };
      return queue;
    });

    // Upload to server
    const { submissionId } = await uploadRecording(zipBlob, get(walletAddress)!);

    // Update status to processing
    uploadQueue.update((queue) => {
      queue[recordingId] = {
        status: 'processing',
        progress: 80,
        submissionId,
        name
      };
      return queue;
    });

    // Start polling status
    pollSubmissionStatus(recordingId, submissionId);
    return true;
  } catch (error) {
    console.error('Failed to upload recording:', error);
    uploadQueue.update((queue) => {
      queue[recordingId] = {
        status: 'failed',
        error: error instanceof Error ? error.message : 'Failed to upload recording'
      };
      return queue;
    });
    return false;
  }
}

// Function to remove an item from the queue
export function removeFromQueue(recordingId: string) {
  uploadQueue.update((queue) => {
    const newQueue = { ...queue };
    delete newQueue[recordingId];
    return newQueue;
  });

  // Clear any interval
  if (statusIntervals[recordingId]) {
    clearInterval(statusIntervals[recordingId]);
    delete statusIntervals[recordingId];
  }
}
