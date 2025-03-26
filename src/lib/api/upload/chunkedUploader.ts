import { apiClient } from '../core/client';
import { get } from 'svelte/store';
import { connectionToken } from '$lib/stores/wallet';
import type { UploadMetadata, UploadProgress } from '$lib/types/upload';

/**
 * Calculates a simple hash of a blob
 * @param blob The blob to hash
 * @returns Promise resolving to hash string
 */
async function calculateHash(blob: Blob): Promise<string> {
  // Convert blob to ArrayBuffer
  const arrayBuffer = await blob.arrayBuffer();
  // Use SubtleCrypto API to calculate SHA-256 hash (more widely supported than MD5)
  const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer);
  // Convert hash to hex string
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  return hashHex;
}

export class ChunkedUploader {
  private uploadId: string | null = null;
  private chunkSize: number = 15 * 1024 * 1024; // Default to 15MB chunks
  private totalChunks: number = 0;
  private uploadedChunks: Set<number> = new Set();
  private abortController: AbortController | null = null;
  private token: string;

  constructor() {
    this.token = get(connectionToken) || '';
  }

  /**
   * Initializes a new chunked upload session
   * @param metadata Metadata about the upload
   * @returns Promise resolving to upload session details
   */
  async initUpload(metadata: UploadMetadata): Promise<{ uploadId: string; expiresIn: number; chunkSize: number }> {
    this.token = get(connectionToken) || '';
    this.abortController = new AbortController();

    const result = await apiClient.post<{ uploadId: string; expiresIn: number; chunkSize: number }>(
      '/forge/upload/init',
      {
        totalChunks: this.totalChunks,
        metadata
      },
      {
        requiresAuth: true,
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );

    this.uploadId = result.uploadId;
    this.chunkSize = result.chunkSize;
    
    return result;
  }

  /**
   * Uploads a single chunk of the file
   * @param chunk The binary chunk data
   * @param chunkIndex Zero-based index of this chunk
   * @returns Promise resolving to upload progress
   */
  async uploadChunk(chunk: Blob, chunkIndex: number): Promise<UploadProgress> {
    if (!this.uploadId) {
      throw new Error('Upload not initialized');
    }

    this.token = get(connectionToken) || '';
    
    // Calculate hash for verification
    const checksum = await calculateHash(chunk);
    
    // Create form data
    const formData = new FormData();
    formData.append('chunk', chunk);
    formData.append('chunkIndex', chunkIndex.toString());
    formData.append('checksum', checksum);

    // Upload the chunk
    const result = await apiClient.post<UploadProgress>(
      `/forge/upload/chunk/${this.uploadId}`,
      formData,
      { requiresAuth: true }
    );

    this.uploadedChunks.add(chunkIndex);
    
    return result;
  }

  /**
   * Gets the current status of an upload
   * @returns Promise resolving to upload status
   */
  async getStatus(): Promise<{ 
    uploadId: string; 
    received: number; 
    total: number; 
    progress: number;
    createdAt: string;
    lastUpdated: string;
  }> {
    if (!this.uploadId) {
      throw new Error('Upload not initialized');
    }

    this.token = get(connectionToken) || '';
    
    return apiClient.get<{
      uploadId: string;
      received: number;
      total: number;
      progress: number;
      createdAt: string;
      lastUpdated: string;
    }>(`/forge/upload/status/${this.uploadId}`, {}, { requiresAuth: true });
  }

  /**
   * Completes the upload process
   * @returns Promise resolving to completion result
   */
  async completeUpload(): Promise<{ 
    message: string; 
    submissionId: string;
    files: Array<{
      file: string;
      s3Key: string;
      size: number;
    }>;
  }> {
    if (!this.uploadId) {
      throw new Error('Upload not initialized');
    }

    this.token = get(connectionToken) || '';
    
    return apiClient.post<{
      message: string;
      submissionId: string;
      files: Array<{
        file: string;
        s3Key: string;
        size: number;
      }>;
    }>(`/forge/upload/complete/${this.uploadId}`, {}, {
      requiresAuth: true,
      headers: {
        'Content-Type': 'application/json'
      }
    });
  }

  /**
   * Cancels an in-progress upload
   * @returns Promise resolving to cancellation result
   */
  async cancelUpload(): Promise<{ message: string }> {
    if (!this.uploadId) {
      throw new Error('Upload not initialized');
    }

    this.token = get(connectionToken) || '';
    
    // Abort any in-progress requests
    if (this.abortController) {
      this.abortController.abort();
      this.abortController = new AbortController();
    }
    
    return apiClient.delete<{ message: string }>(`/forge/upload/cancel/${this.uploadId}`, { requiresAuth: true });
  }

  /**
   * Splits a blob into chunks of the specified size
   * @param blob The blob to split
   * @returns Array of chunks
   */
  splitIntoChunks(blob: Blob): { chunks: Blob[]; totalChunks: number } {
    const chunks: Blob[] = [];
    let start = 0;
    
    while (start < blob.size) {
      const end = Math.min(start + this.chunkSize, blob.size);
      chunks.push(blob.slice(start, end));
      start = end;
    }
    
    this.totalChunks = chunks.length;
    return { chunks, totalChunks: chunks.length };
  }
}
