import { UploadManager } from '$lib/utils/uploadManager';
import { get, writable, type Writable } from 'svelte/store';
import { walletAddress } from './wallet';

/**
 * Creates a store for the upload manager
 */
function createUploadManagerStore(): Writable<UploadManager> {
  const store = writable<UploadManager>(new UploadManager(() => get(walletAddress)));

  return store;
}

export const uploadManager = createUploadManagerStore();
