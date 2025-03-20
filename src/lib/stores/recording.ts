import { RecordingState } from '$lib/types/gym';
import { writable } from 'svelte/store';

export const recordingState = writable<RecordingState>(RecordingState.off);
