import { invoke } from '@tauri-apps/api/core';
import { uploadRecording, getSubmissionStatus } from '$lib/api/forge';

// Define event types
type UploadEventType = 'queueUpdate' | 'statusChange';

// Define event callback types
type QueueUpdateCallback = (recordingId: string, item: UploadQueueItem) => void;
type EventCallback = QueueUpdateCallback; // Can expand with more specific types if needed

type UploadQueueItem = {
  status: 'queued' | 'uploading' | 'zipping' | 'processing' | 'completed' | 'failed';
  progress?: number;
  error?: string;
  submissionId?: string;
  name?: string;
};

type UploadQueue = {
  [recordingId: string]: UploadQueueItem;
};

export class UploadManager {
  private uploadQueue: UploadQueue = {};
  private statusIntervals: { [recordingId: string]: number } = {};
  private subscribers: Array<(queue: UploadQueue) => void> = [];
  private eventListeners: Map<UploadEventType, Map<string, Set<EventCallback>>> = new Map();

  constructor(private getWalletAddress: () => string | null) {}

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
    if (event === 'queueUpdate' && recordingId !== '*' && this.uploadQueue[recordingId]) {
      callback(recordingId, this.uploadQueue[recordingId]);
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
      specificCallbacks.forEach(callback => callback(recordingId, data));
    }
    
    // Call wildcard listeners that listen to all recording IDs
    const wildcardCallbacks = recordingsMap.get('*');
    if (wildcardCallbacks) {
      wildcardCallbacks.forEach(callback => callback(recordingId, data));
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
      await invoke('get_upload_data_allowed', { confirmed: allowed });
    } catch (error) {
      console.error('Failed to save upload allowance:', error);
    }
  }

  /**
   * Checks if there are any active uploads in the queue
   */
  hasActiveUploads(): boolean {
    return Object.values(this.uploadQueue).some(
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
            progress: 100
          });
          // successful upload, remove interval
          clearInterval(this.statusIntervals[recordingId]);
          delete this.statusIntervals[recordingId];

          // // Auto-remove completed uploads after 5 seconds
          // setTimeout(() => {
          //   this.removeFromQueue(recordingId);
          // }, 5000);
        } else if (status.status === 'failed') {
          this.updateQueue(recordingId, {
            status: 'failed',
            error: status.error || 'Upload failed'
          });
          // upload failed, remove intervals
          clearInterval(this.statusIntervals[recordingId]);
          delete this.statusIntervals[recordingId];
          // todo: allow for retry
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
   * Handles the upload process for a recording
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

      // Update progress
      this.updateQueue(recordingId, {
        status: 'uploading',
        name,
        progress: 30
      });

      // Convert to Blob
      const zipBlob = new Blob([Uint8Array.from(zipBytes)], { type: 'application/zip' });

      // Update progress
      this.updateQueue(recordingId, {
        status: 'uploading',
        name,
        progress: 60
      });

      // Upload to server
      const { submissionId } = await uploadRecording(zipBlob, walletAddress);

      // Update status to processing
      this.updateQueue(recordingId, {
        status: 'processing',
        progress: 80,
        submissionId,
        name
      });

      // Start polling status until processing is done
      this.pollSubmissionStatus(recordingId, submissionId);
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
    const newQueue = { ...this.uploadQueue };
    delete newQueue[recordingId];
    this.uploadQueue = newQueue;
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
    const previousStatus = this.uploadQueue[recordingId]?.status;
    
    this.uploadQueue = {
      ...this.uploadQueue,
      [recordingId]: {
        ...this.uploadQueue[recordingId],
        ...updates
      }
    };
    
    // Get the updated item
    const updatedItem = this.uploadQueue[recordingId];
    
    // Notify general subscribers (keep existing behavior)
    this.notifySubscribers();
    
    // Emit queueUpdate event
    this.emit('queueUpdate', recordingId, updatedItem);
    
    // Emit statusChange event if status changed
    if (previousStatus !== updatedItem.status) {
      this.emit('statusChange', recordingId, updatedItem);
    }
  }

  /**
   * Wait for a recording to reach a terminal status (completed or failed)
   * @param recordingId The recording ID to wait for
   * @returns A promise that resolves with the final status or rejects with an error
   */
  waitForCompletion(recordingId: string): Promise<UploadQueueItem> {
    return new Promise((resolve, reject) => {
      // If the recording doesn't exist, reject immediately
      if (!this.uploadQueue[recordingId]) {
        reject(new Error(`No upload found for recording ID: ${recordingId}`));
        return;
      }
      
      // If already in a terminal state, resolve/reject immediately
      const currentStatus = this.uploadQueue[recordingId];
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
    callback(this.uploadQueue);

    // Return unsubscribe function
    return () => {
      this.subscribers = this.subscribers.filter((sub) => sub !== callback);
    };
  }

  /**
   * Notifies all subscribers of queue changes
   */
  private notifySubscribers(): void {
    this.subscribers.forEach((callback) => callback(this.uploadQueue));
  }

  /**
   * Gets the current upload queue state
   */
  get getUploadQueue(): UploadQueue {
    return this.uploadQueue;
  }
}
