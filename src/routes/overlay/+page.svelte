<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import * as gym from '$lib/gym';

  type RecordingState = 'recording' | 'stopping' | 'stopped';

  interface RecordingStatus {
    state: RecordingState;
  }

  interface QuestOverlayEvent {
    quest: Quest | null;
  }

  let recordingState = $state<RecordingState>('stopped');
  let recordingTime = $state(0);
  let timer: number;

  interface Quest {
    task_id: string;
    title: string;
    original_instruction: string;
    concrete_scenario: string;
    objective: string;
    relevant_applications: string[];
    subgoals: string[];
  }

  let currentQuest = $state<Quest | null>(null);

  function startTimer() {
    timer = setInterval(() => {
      recordingTime++;
    }, 1000);
  }

  function stopTimer() {
    if (timer) clearInterval(timer);
  }

  function formatTime(seconds: number) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  function resetQuest() {
    recordingTime = 0;
    currentQuest = null;
  }

  function initializeQuest(quest: Quest) {
    currentQuest = quest;
  }

  onMount(() => {
    let unlistenRecording: () => void;
    let unlistenQuest: () => void;

    // Listen for recording status changes
    listen<RecordingStatus>('recording-status', (event) => {
      recordingState = event.payload.state;

      if (event.payload.state === 'recording') {
        startTimer();
      } else if (event.payload.state === 'stopped') {
        stopTimer();
        resetQuest();
      }
    }).then((unlistenFn) => {
      unlistenRecording = unlistenFn;
    });

    // Listen for quest updates
    listen<QuestOverlayEvent>('quest-overlay', (event) => {
      if (event.payload.quest) {
        initializeQuest(event.payload.quest);
      } else {
        currentQuest = null;
      }
    }).then((unlistenFn) => {
      unlistenQuest = unlistenFn;
    });

    return () => {
      unlistenRecording?.();
      unlistenQuest?.();
    };
  });

  onDestroy(() => {
    stopTimer();
  });
</script>

<div class="overlay-content">
  {#if recordingState !== 'stopped' && currentQuest}
    <div
      class="fixed top-4 right-4 w-96 bg-black/80 rounded-lg border border-purple-500/50 p-4 text-white shadow-lg backdrop-blur-sm">
      <div class="flex items-center justify-between mb-3">
        <span class="px-2 py-1 text-xs font-semibold bg-red-600 rounded-full animate-pulse">
          {#if recordingState === 'recording'}
            Recording
          {:else if recordingState === 'stopping'}
            Saving...
          {/if}
        </span>
        <span class="text-purple-400 font-mono">{formatTime(recordingTime)}</span>
      </div>

      <p class="text-sm text-purple-400 mb-4">
        {currentQuest.objective}
      </p>

      {#if currentQuest.relevant_applications.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each currentQuest.relevant_applications as app}
            <span class="px-2 py-1 text-xs bg-purple-600/20 rounded-full">{app}</span>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
  }
</style>
