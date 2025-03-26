<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ChevronDown, ChevronUp, LoaderCircle, Lock, Square, Unlock } from 'lucide-svelte';
  import { RecordingState, type Quest } from '$lib/types/gym';
  import { stopRecording } from '$lib/api/endpoints/gym';
  import { slide } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';

  interface QuestOverlayEvent {
    quest: Quest | null;
  }
  // Window
  const appWindow = getCurrentWindow();

  // Recording State
  let recordingTime = $state(0);
  let currentQuest = $state<Quest | null>(null);
  let recordingState: RecordingState = $state(RecordingState.starting);

  // Time tracking
  let hours = $state(0);
  let minutes = $state(0);
  let seconds = $state(0);
  let timerInterval: number | undefined;

  // UI State
  let focused = $state(false);
  let isLocked = $state(false);
  let isCollapsed = $state(false);

  // Dragging state
  let overlayElement: HTMLDivElement | undefined = $state();

  const formattedTime = $derived(
    `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
  );

  function startTimer() {
    if (!timerInterval) {
      console.log('starting the timer');
      timerInterval = setInterval(() => {
        seconds++;
        if (seconds >= 60) {
          seconds = 0;
          minutes++;
          if (minutes >= 60) {
            minutes = 0;
            hours++;
          }
        }
      }, 1000);
    }
  }

  function stopTimer() {
    console.log('stopping the timer');
    clearInterval(timerInterval);
    timerInterval = undefined;
  }

  function handleMouseDown(e: MouseEvent) {
    if (isLocked) return;
    if (e.buttons === 1) {
      appWindow.startDragging();
    }
  }

  function initializeQuest(quest: Quest) {
    currentQuest = quest;
  }

  onMount(() => {
    let unlistenRecording: () => void;
    let unlistenQuest: () => void;

    // get recording status
    invoke<RecordingState>('get_recording_state').then((payload) => {
      recordingState = payload;
      if (recordingState === RecordingState.recording) {
        startTimer();
      } else if (recordingState === RecordingState.saving) {
        stopTimer();
      }
    });

    // Get current quest data if available
    invoke<Quest | null>('get_current_quest')
      .then((quest) => {
        if (quest) {
          initializeQuest(quest);
        }
      })
      .catch((error) => {
        console.error('Failed to get current quest:', error);
      });

    // Listen for recording status changes
    listen<{ state: RecordingState }>('recording-status', (event) => {
      recordingState = event.payload.state;
      if (event.payload.state === RecordingState.recording) {
        startTimer();
      } else if (event.payload.state === RecordingState.saving) {
        stopTimer();
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

    appWindow.onFocusChanged(() => (focused = !focused));
    return () => {
      unlistenRecording?.();
      unlistenQuest?.();
    };
  });

  onDestroy(() => {
    stopTimer();
  });

  // Handle button events
  async function handleStop() {
    if (recordingState === RecordingState.recording) await stopRecording('done');
  }

  function toggleCollapsed() {
    isCollapsed = !isCollapsed;
  }

  function toggleLocked() {
    isLocked = !isLocked;
  }
</script>

{#if recordingState !== RecordingState.off}
  <div
    bind:this={overlayElement}
    role="dialog"
    aria-label="Recording overlay"
    class="shadow-lg select-none transition-opacity duration-200 z-50 w-screen h-auto absolute rounded-lg overflow-hidden border-secondary-400">
    <!-- Header with recording indicator - serves as drag handle -->
    <div
      role="button"
      tabindex="0"
      class="p-3 flex {focused
        ? 'opacity-100'
        : 'opacity-70'} items-center justify-between bg-primary-600 {isLocked
        ? 'cursor-default'
        : 'cursor-move'}"
      onmousedown={handleMouseDown}>
      <div class="flex items-center">
        {#if recordingState == RecordingState.recording || recordingState == RecordingState.starting}
          <div class="w-3 h-3 rounded-full bg-red-500 animate-pulse mr-2"></div>
          <span class="text-accent-100 font-title text-sm">
            {recordingState === RecordingState.starting ? 'Starting' : 'Recording'}
          </span>
          <!-- PAUSE STATE
          {:else if isPaused}
            <div class="w-3 h-3 rounded-full bg-yellow-500 mr-2"></div>
            <span class="text-accent-100 font-title text-sm">Paused</span> -->
        {:else if recordingState == RecordingState.saving}
          <div class="w-3 h-3 rounded-full bg-emerald-500 animate-pulse mr-2"></div>
          <span class="text-accent-100 font-title text-sm select-none">Saving</span>
        {:else}
          <div class="w-3 h-3 rounded-full bg-gray-500 mr-2"></div>
          <span class="text-accent-100 font-title text-sm select-none">Stopped</span>
        {/if}
      </div>

      <div class="flex items-center">
        <span class="text-accent-100 select-none text-xs mr-2">{formattedTime}</span>

        <!-- Collapse/Expand toggle -->
        <button
          onclick={toggleCollapsed}
          class="text-accent-100 {focused
            ? 'opacity-90'
            : 'opacity-70'} hover:opacity-100 ml-1 transition-opacity">
          {#if isCollapsed}
            <ChevronUp class="w-4 h-4" />
          {:else}
            <ChevronDown class="w-4 h-4" />
          {/if}
        </button>

        <!-- Lock/Unlock toggle -->
        <button
          onclick={toggleLocked}
          class="text-accent-100 {focused
            ? 'opacity-90'
            : 'opacity-70'} hover:opacity-100 ml-1 transition-opacity">
          {#if isLocked}
            <Lock class="w-4 h-4" />
          {:else}
            <Unlock class="w-4 h-4" />
          {/if}
        </button>
      </div>
    </div>

    <!-- Content area - hidden when collapsed -->
    {#if !isCollapsed}
      <div class="flex flex-col select-none" transition:slide>
        <!-- Task details section -->
        <div class="p-2 {focused ? 'bg-primary-400/100' : 'bg-primary-400/50'}">
          {#if currentQuest}
            <h3 class="text-secondary-200 font-title text-sm mb-2 drop-shadow">
              {currentQuest.title}
            </h3>
            <ul
              class="list-disc overflow-y-auto max-h-[120px] pl-5 text-accent-100 text-xs space-y-1">
              {#each currentQuest.objectives || [] as objective}
                {#if objective.includes('<app>')}
                  <li class="drop-shadow">
                    {#if objective.startsWith('<app>')}
                      <span
                        class="bg-primary-500 text-primary-800 px-1 rounded text-xs font-semibold">
                        {objective.match(/<app>(.*?)<\/app>/)?.[1] || ''}
                      </span>
                      {objective.replace(/<app>.*?<\/app>/, '').trim()}
                    {:else}
                      {objective.split('<app>')[0].trim()}
                      <span
                        class="bg-primary-500 text-primary-800 px-1 rounded text-xs font-semibold">
                        {objective.match(/<app>(.*?)<\/app>/)?.[1] || ''}
                      </span>
                      {objective.split('</app>')[1]?.trim() || ''}
                    {/if}
                  </li>
                {:else}
                  <li class="drop-shadow">{objective}</li>
                {/if}
              {/each}
            </ul>
          {:else}
            <LoaderCircle class="w-5 h-5 mx-auto animate-spin text-white" />
          {/if}
        </div>

        <!-- Control buttons -->
        <div
          class="flex border-t mt-auto border-primary-300 bg-primary-300 {focused
            ? 'opacity-100'
            : 'opacity-90'}">
          {#if recordingState == RecordingState.recording}
            <button
              onclick={handleStop}
              class="flex-1 py-2 px-3 flex items-center justify-center text-accent-100 hover:bg-primary-400 transition-all duration-200 hover:text-red-500">
              <Square class="w-4 h-4 mr-1" />
              <span class="text-xs">Stop</span>
            </button>
          {:else if recordingState == RecordingState.starting || recordingState == RecordingState.saving}
            <button
              class="flex-1 py-2 px-3 flex items-center justify-center text-accent-100 hover:bg-primary-400 transition-all duration-200 hover:text-red-500">
              <LoaderCircle class="w-4 h-4 animate-spin" />
            </button>
          {/if}

          <!-- PAUSE BUTTON
          <div class="border-r border-primary-300"></div>

          <button
            on:click={handlePause}
            class="flex-1 py-2 px-3 flex items-center justify-center text-accent-100 hover:bg-primary-300 transition-all duration-200 hover:text-yellow-300">
            {#if !isPaused}
              <Pause class="h-4 w-4 mr-1" />
              <span class="text-xs">Pause</span>
            {:else}
              <Play class="h-4 w-4 mr-1" />
              <span class="text-xs">Resume</span>
            {/if}
          </button> -->
        </div>
      </div>
    {/if}
  </div>
{/if}
