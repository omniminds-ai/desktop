import type { SubmissionStatus } from './forge';

/**
 * Upload queue item representing a recording being uploaded
 */
export type UploadQueueItem = {
  status: 'queued' | 'uploading' | 'zipping' | 'processing' | 'completed' | 'failed';
  progress?: number;
  error?: string;
  submissionId?: string;
  name?: string;
  result?: SubmissionStatus;
  uploadId?: string;
  currentChunk?: number;
  totalChunks?: number;
  uploadedBytes?: number;
  totalBytes?: number;
};

/**
 * Upload queue mapping recording IDs to queue items
 */
export type UploadQueue = {
  [recordingId: string]: UploadQueueItem;
};

/**
 * Upload progress information
 */
export interface UploadProgress {
  uploadId: string;
  chunkIndex: number;
  received: number;
  total: number;
  progress: number;
}

/**
 * Metadata for an upload
 */
export interface UploadMetadata {
  poolId?: string;
  generatedTime?: number;
  id: string;
}

/**
 * Event types for the upload system
 */
export type UploadEventType = 'queueUpdate' | 'statusChange';

/**
 * Callback for queue update events
 */
export type QueueUpdateCallback = (recordingId: string, item: UploadQueueItem) => void;

/**
 * Generic event callback type
 */
export type EventCallback = QueueUpdateCallback; // Can expand with more specific types if needed
