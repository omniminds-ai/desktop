import { UploadManager } from '$lib/uploadManager';
import { get, writable } from 'svelte/store';
import { walletAddress } from './wallet';

export const uploadManager = writable<UploadManager>(new UploadManager(() => get(walletAddress)));
