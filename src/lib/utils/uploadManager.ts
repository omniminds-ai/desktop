import { invoke } from '@tauri-apps/api/core';
import { getSubmissionStatus } from '$lib/api/forge';
import { writable, type Writable, get } from 'svelte/store';
import type { SubmissionStatus } from '../types/forge';
import { ChunkedUploader } from '../api/chunkedUploader';
import type { UploadQueue, UploadQueueItem, UploadEventType, EventCallback } from '../types/upload';

export class UploadManager {
  private queueStore: Writable<UploadQueue> = writable({});
  private statusIntervals: { [recordingId: string]: number } = {};
  private subscribers: Array<(queue: UploadQueue) => void> = [];
  private eventListeners: Map<UploadEventType, Map<string, Set<EventCallback>>> = new Map();

  constructor(private getWalletAddress: () => string | null) {}

  /**
   * Expose the queue store directly for reactive access
   */
  get queue() {
    return this.queueStore;
  }

  /**
   * Register an event listener
   * @param event The event type to listen for
   * @param recordingId The specific recording ID to listen for, or '*' for all recordings
   * @param callback The callback function to execute when the event occurs
   * @returns A function to remove this specific listener
   */
  on(event: UploadEventType, recordingId: string, callback: EventCallback): () => void {
    // Initialize event type map if it doesn't exist
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Map());
    }

    const recordingsMap = this.eventListeners.get(event)!;

    // Initialize recording ID set if it doesn't exist
    if (!recordingsMap.has(recordingId)) {
      recordingsMap.set(recordingId, new Set());
    }

    const callbacks = recordingsMap.get(recordingId)!;
    callbacks.add(callback);

    // If this is a queueUpdate event and the recording already exists, trigger immediately
    const currentQueue = get(this.queueStore);
    if (event === 'queueUpdate' && recordingId !== '*' && currentQueue[recordingId]) {
      callback(recordingId, currentQueue[recordingId]);
    }

    // Return unsubscribe function
    return () => {
      const recordingsMap = this.eventListeners.get(event);
      if (!recordingsMap) return;

      const callbacks = recordingsMap.get(recordingId);
      if (!callbacks) return;

      callbacks.delete(callback);

      // Clean up empty sets and maps
      if (callbacks.size === 0) {
        recordingsMap.delete(recordingId);
      }

      if (recordingsMap.size === 0) {
        this.eventListeners.delete(event);
      }
    };
  }

  /**
   * Emit an event to all registered listeners
   * @param event The event type to emit
   * @param recordingId The recording ID associated with the event
   * @param data The data to pass to the listeners
   */
  private emit(event: UploadEventType, recordingId: string, data: UploadQueueItem): void {
    const recordingsMap = this.eventListeners.get(event);
    if (!recordingsMap) return;

    // Call specific listeners for this recording ID
    const specificCallbacks = recordingsMap.get(recordingId);
    if (specificCallbacks) {
      specificCallbacks.forEach((callback) => callback(recordingId, data));
    }

    // Call wildcard listeners that listen to all recording IDs
    const wildcardCallbacks = recordingsMap.get('*');
    if (wildcardCallbacks) {
      wildcardCallbacks.forEach((callback) => callback(recordingId, data));
    }
  }

  /**
   * Checks if uploading data is allowed based on user preferences
   */
  async isUploadDataAllowed(): Promise<boolean> {
    try {
      const confirmed = await invoke<boolean>('get_upload_data_allowed');
      return confirmed;
    } catch (error) {
      console.error('Failed to check upload confirmation allowance:', error);
      return false;
    }
  }

  /**
   * Sets the user preference for allowing data uploads
   */
  async setUploadDataAllowed(allowed: boolean): Promise<void> {
    try {
      console.log(allowed);
      await invoke('set_upload_data_allowed', { confirmed: allowed });
    } catch (error) {
      console.error('Failed to save upload allowance:', error);
    }
  }

  /**
   * Checks if there are any active uploads in the queue
   */
  hasActiveUploads(): boolean {
    const queue = get(this.queueStore);
    return Object.values(queue).some(
      (item) =>
        item.status === 'queued' || item.status === 'uploading' || item.status === 'processing'
    );
  }

  /**
   * Cleans up all active upload intervals
   */
  cleanupIntervals(): void {
    Object.values(this.statusIntervals).forEach((interval) => {
      clearInterval(interval);
    });
    this.statusIntervals = {};
  }

  /**
   * Polls the submission status from the server
   */
  private pollSubmissionStatus(recordingId: string, submissionId: string): void {
    if (this.statusIntervals[recordingId]) {
      clearInterval(this.statusIntervals[recordingId]);
    }

    this.statusIntervals[recordingId] = setInterval(async () => {
      try {
        const status = await getSubmissionStatus(submissionId);

        if (status.status === 'completed') {
          this.updateQueue(recordingId, {
            status: 'completed',
            progress: 100,
            result: status
          });
          // successful upload, remove interval
          clearInterval(this.statusIntervals[recordingId]);
          delete this.statusIntervals[recordingId];
        } else if (status.status === 'failed') {
          this.updateQueue(recordingId, {
            status: 'failed',
            error: status.error || 'Upload failed'
          });
          // upload failed, remove intervals
          clearInterval(this.statusIntervals[recordingId]);
          delete this.statusIntervals[recordingId];
        } else if (status.status === 'processing') {
          // Set arbitrary progress for processing state
          this.updateQueue(recordingId, {
            status: 'processing',
            progress: 80
          });
        }
      } catch (error) {
        console.error('Failed to get submission status:', error);

        this.updateQueue(recordingId, {
          status: 'failed',
          error: error instanceof Error ? error.message : 'Failed to check status'
        });

        clearInterval(this.statusIntervals[recordingId]);
        delete this.statusIntervals[recordingId];
      }
    }, 5000);
  }

  /**
   * Handles the upload process for a recording using chunked uploads
   */
  async handleUpload(recordingId: string, name: string): Promise<boolean> {
    const walletAddress = this.getWalletAddress();

    if (!walletAddress) {
      this.updateQueue(recordingId, {
        status: 'failed',
        error: 'Please connect your wallet first.'
      });
      return false;
    }

    // Check if upload has been confirmed
    const isConfirmed = await this.isUploadDataAllowed();
    if (!isConfirmed) {
      // Return false to indicate that confirmation is needed
      return false;
    }

    // Add to queue
    this.updateQueue(recordingId, {
      status: 'queued',
      name
    });

    try {
      // Update status to zipping
      this.updateQueue(recordingId, {
        status: 'zipping',
        name,
        progress: 0
      });

      // Get zip file as bytes
      const zipBytes = await invoke<number[]>('create_recording_zip', { recordingId });

      // Convert to Blob
      const zipBlob = new Blob([Uint8Array.from(zipBytes)], { type: 'application/zip' });

      // Update progress after zipping
      this.updateQueue(recordingId, {
        status: 'uploading',
        name,
        progress: 10,
        totalBytes: zipBlob.size,
        uploadedBytes: 0
      });

      // Initialize chunked uploader
      const uploader = new ChunkedUploader();

      // Split the blob into chunks
      const { chunks, totalChunks } = uploader.splitIntoChunks(zipBlob);

      // Update queue with total chunks info
      this.updateQueue(recordingId, {
        totalChunks,
        currentChunk: 0
      });

      // Initialize upload
      const initResult = await uploader.initUpload({
        id: recordingId
      });

      // Store upload ID
      this.updateQueue(recordingId, {
        uploadId: initResult.uploadId
      });

      // Upload each chunk
      for (let i = 0; i < chunks.length; i++) {
        // Calculate uploaded bytes
        const uploadedBytes = chunks.slice(0, i).reduce((total, chunk) => total + chunk.size, 0);

        // Update current chunk index and uploaded bytes
        this.updateQueue(recordingId, {
          currentChunk: i,
          uploadedBytes: uploadedBytes,
          // Calculate progress: 10% for zipping + 70% for uploading chunks
          progress: 10 + Math.round((i / totalChunks) * 70)
        });

        // Upload the chunk
        await uploader.uploadChunk(chunks[i], i);

        // After successful upload, add this chunk's size to uploaded bytes
        this.updateQueue(recordingId, {
          uploadedBytes: uploadedBytes + chunks[i].size
        });
      }

      // Complete the upload
      const completeResult = await uploader.completeUpload();

      // Update status to processing
      this.updateQueue(recordingId, {
        status: 'processing',
        progress: 80,
        submissionId: completeResult.submissionId,
        name
      });

      // Start polling status until processing is done
      this.pollSubmissionStatus(recordingId, completeResult.submissionId);
      return true;
    } catch (error) {
      console.error('Failed to upload recording:', error);

      this.updateQueue(recordingId, {
        status: 'failed',
        error: error instanceof Error ? error.message : 'Failed to upload recording'
      });

      return false;
    }
  }

  /**
   * Removes an item from the upload queue
   */
  removeFromQueue(recordingId: string): void {
    this.queueStore.update((queue) => {
      const newQueue = { ...queue };
      delete newQueue[recordingId];
      return newQueue;
    });

    this.notifySubscribers();

    // Clear any interval
    if (this.statusIntervals[recordingId]) {
      clearInterval(this.statusIntervals[recordingId]);
      delete this.statusIntervals[recordingId];
    }
  }

  /**
   * Updates a queue item with new properties
   */
  private updateQueue(recordingId: string, updates: Partial<UploadQueueItem>): void {
    let updatedItem: UploadQueueItem;
    let previousStatus: string | undefined;

    this.queueStore.update((queue) => {
      previousStatus = queue[recordingId]?.status;

      const updatedQueue = {
        ...queue,
        [recordingId]: {
          ...queue[recordingId],
          ...updates
        }
      };

      updatedItem = updatedQueue[recordingId];

      return updatedQueue;
    });

    // Notify general subscribers (keep existing behavior)
    this.notifySubscribers();

    // Emit queueUpdate event
    this.emit('queueUpdate', recordingId, updatedItem!);

    // Emit statusChange event if status changed
    if (previousStatus !== updatedItem!.status) {
      this.emit('statusChange', recordingId, updatedItem!);
    }
  }

  /**
   * Wait for a recording to reach a terminal status (completed or failed)
   * @param recordingId The recording ID to wait for
   * @returns A promise that resolves with the final status or rejects with an error
   */
  waitForCompletion(recordingId: string): Promise<UploadQueueItem> {
    return new Promise((resolve, reject) => {
      const currentQueue = get(this.queueStore);

      // If the recording doesn't exist, reject immediately
      if (!currentQueue[recordingId]) {
        reject(new Error(`No upload found for recording ID: ${recordingId}`));
        return;
      }

      // If already in a terminal state, resolve/reject immediately
      const currentStatus = currentQueue[recordingId];
      if (currentStatus.status === 'completed') {
        resolve(currentStatus);
        return;
      }
      if (currentStatus.status === 'failed') {
        reject(new Error(currentStatus.error || 'Upload failed'));
        return;
      }

      // Otherwise, wait for status to change to a terminal state
      const unsubscribe = this.on('queueUpdate', recordingId, (_, status) => {
        if (status.status === 'completed') {
          unsubscribe();
          resolve(status);
        } else if (status.status === 'failed') {
          unsubscribe();
          reject(new Error(status.error || 'Upload failed'));
        }
      });
    });
  }

  /**
   * Subscribes to queue changes
   */
  subscribe(callback: (queue: UploadQueue) => void): () => void {
    this.subscribers.push(callback);
    // Immediately notify with current state
    callback(get(this.queueStore));

    // Return unsubscribe function
    return () => {
      this.subscribers = this.subscribers.filter((sub) => sub !== callback);
    };
  }

  /**
   * Notifies all subscribers of queue changes
   */
  private notifySubscribers(): void {
    const currentQueue = get(this.queueStore);
    this.subscribers.forEach((callback) => callback(currentQueue));
  }
}
