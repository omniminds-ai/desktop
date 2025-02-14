<script lang="ts">
  import { platform } from '@tauri-apps/plugin-os';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import * as gym from '$lib/gym';
  import AppText from '$lib/components/gym/AppText.svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';

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
  import type { Quest } from '$lib/types/gym';
  let currentQuest = $state<Quest | null>(null);
  
  const slideIn = tweened(-100, {
    duration: 500,
    easing: cubicOut
  });

  const showContent = tweened(1, {
    duration: 300,
    easing: cubicOut
  });

  function startAnimations() {
    slideIn.set(0);
    setTimeout(() => {
      showContent.set(0);
    }, 3000);
  }

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
    showContent.set(1);
    startAnimations();
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

  let isMinimized = $derived($showContent === 0);
</script>

<div class="overlay-content">
  {#if recordingState !== 'stopped' && currentQuest}
    <div 
      class="fixed top-4 right-0 bg-black/80 rounded-lg border border-purple-500/50 shadow-lg backdrop-blur-sm overflow-hidden transition-all duration-300"
      style="transform: translateX(calc({$slideIn}px))">
      <div class="p-2">
        <div class="flex items-center gap-1.5">
          <span class="px-2 py-1 text-xs font-semibold bg-red-600 rounded-full animate-pulse whitespace-nowrap">
            {#if recordingState === 'recording'}
              Recording
            {:else if recordingState === 'stopping'}
              Saving...
            {/if}
          </span>
          <span class="text-purple-400 font-mono transition-opacity duration-300" class:hidden={isMinimized}>{formatTime(recordingTime)}</span>
        </div>

        <div class="transition-opacity duration-300" class:hidden={isMinimized}>
          <h3 class="text-lg font-bold text-white my-2 break-words">{currentQuest.title}</h3>
          
          <div class="mb-3">
            <h4 class="text-sm font-semibold text-white/90 mb-1">Objectives:</h4>
            <ul class="list-disc pl-4 space-y-1">
              {#each currentQuest.objectives as objective}
                <li class="text-sm text-white/80 break-words">
                  <AppText text={objective} />
                </li>
              {/each}
            </ul>
          </div>
        </div>
      </div>
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
