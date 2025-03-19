<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { ChevronDown, ChevronUp, Lock, Pause, Play, Square, Unlock } from 'lucide-svelte';
  import type { Quest } from '$lib/types/gym';
  import { stopRecording } from '$lib/gym';
  import { slide } from 'svelte/transition';

  type RecordingState = 'recording' | 'stopping' | 'stopped';

  interface RecordingStatus {
    state: RecordingState;
  }

  interface QuestOverlayEvent {
    quest: Quest | null;
  }

  // Recording State
  let recordingState = $state<RecordingState>('stopped');
  let recordingTime = $state(0);
  let currentQuest = $state<Quest | null>(null);

  // Time tracking
  let hours = 0;
  let minutes = 0;
  let seconds = 0;
  let timerInterval: number | undefined;

  // UI State
  let isHovered = $state(false);
  let isLocked = $state(false);
  let isCollapsed = $state(false);

  // Dragging state
  let position = $state({ x: 0, y: 20 });
  let isDragging = $state(false);
  let dragOffset = $state({ x: 0, y: 0 });
  let overlayElement: HTMLDivElement | undefined = $state();

  const formattedTime = $derived(
    `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
  );

  function startTimer() {
    if (!timerInterval && recordingState === 'recording') {
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
    clearInterval(timerInterval);
    timerInterval = undefined;
  }

  $effect(() => {
    if (recordingState === 'recording') {
      startTimer();
    } else {
      stopTimer();
    }
  });

  function handleMouseDown(e: MouseEvent) {
    if (isLocked) return;
    if (!overlayElement) return;

    isDragging = true;
    const rect = overlayElement.getBoundingClientRect();
    dragOffset = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    };
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!overlayElement) return;
    if (isDragging && !isLocked) {
      // Calculate new position, keeping the overlay within viewport bounds
      const newX = Math.max(
        0,
        Math.min(e.clientX - dragOffset.x, window.innerWidth - overlayElement.offsetWidth)
      );
      const newY = Math.max(
        0,
        Math.min(e.clientY - dragOffset.y, window.innerHeight - overlayElement.offsetHeight)
      );

      position = { x: newX, y: newY };
    }
  }

  function handleMouseUp() {
    isDragging = false;
  }

  function resetQuest() {
    recordingTime = 0;
    currentQuest = null;
  }

  function initializeQuest(quest: Quest) {
    currentQuest = quest;
  }

  onMount(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
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
      alert(currentQuest);
      alert(recordingState);
    });

    return () => {
      unlistenRecording?.();
      unlistenQuest?.();
    };
  });

  onDestroy(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    stopTimer();
  });

  // Handle button events
  async function handleStop() {
    //todo: stop the recording
    await stopRecording();
  }

  function toggleCollapsed() {
    isCollapsed = !isCollapsed;
  }

  function toggleLocked() {
    isLocked = !isLocked;
  }

  function handleMouseEnter() {
    isHovered = true;
  }

  function handleMouseLeave() {
    isHovered = false;
    console.log(window.innerWidth - 280);
  }
</script>

<!-- {#if recordingState !== 'stopped' && currentQuest} -->
{#if true}
  <div
    bind:this={overlayElement}
    role="dialog"
    aria-label="Recording overlay"
    style="left: {position.x}px; 
         top: {position.y}px; 
         opacity: {isHovered ? 0.95 : 0.7};"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
    class="shadow-lg transition-opacity duration-200 z-50 w-[280px] max-h-[203px] absolute rounded-lg overflow-hidden bg-primary-500 border border-secondary-400">
    <!-- Header with recording indicator - serves as drag handle -->
    <div
      role="button"
      tabindex="0"
      class="p-3 flex items-center justify-between bg-primary-600 {isLocked
        ? 'cursor-default'
        : 'cursor-move'}"
      onmousedown={handleMouseDown}>
      <div class="flex items-center">
        {#if recordingState == 'recording'}
          <div class="w-3 h-3 rounded-full bg-red-500 animate-pulse mr-2"></div>
          <span class="text-accent-100 font-title text-sm">Recording</span>
          <!-- PAUSE STATE
          {:else if isPaused}
            <div class="w-3 h-3 rounded-full bg-yellow-500 mr-2"></div>
            <span class="text-accent-100 font-title text-sm">Paused</span> -->
        {:else}
          <div class="w-3 h-3 rounded-full bg-accent-500 mr-2"></div>
          <span class="text-accent-100 font-title text-sm">Stopped</span>
        {/if}
      </div>

      <div class="flex items-center">
        <span class="text-accent-100 text-xs mr-2">{formattedTime}</span>

        <!-- Collapse/Expand toggle -->
        <button
          onclick={toggleCollapsed}
          class="text-accent-100 opacity-70 hover:opacity-100 ml-1 transition-opacity">
          {#if isCollapsed}
            <ChevronUp class="w-4 h-4" />
          {:else}
            <ChevronDown class="w-4 h-4" />
          {/if}
        </button>

        <!-- Lock/Unlock toggle -->
        <button
          onclick={toggleLocked}
          class="text-accent-100 opacity-70 hover:opacity-100 ml-1 transition-opacity">
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
      <div class="flex flex-col" transition:slide>
        <!-- Task details section -->
        <div class="p-3 bg-primary-400">
          <h3 class="text-secondary-200 font-title text-sm mb-2">{currentQuest?.title}</h3>
          <ul class="list-disc overflow-y-scroll pl-5 text-accent-100 text-xs space-y-1">
            {#each currentQuest?.objectives || [] as objective}
              <li>{objective}</li>
            {/each}
            <li>aaaa</li>
            <li>aaaa</li>
            <li>aaaa</li>
            <li>aaaa</li>
          </ul>
        </div>

        <!-- Control buttons -->
        <div class="flex border-t mt-auto border-primary-300">
          <button
            onclick={handleStop}
            class="flex-1 py-2 px-3 flex items-center justify-center text-accent-100 hover:bg-primary-300 transition-all duration-200 hover:text-red-300">
            <Square class="w-4 h-4 mr-1" />
            <span class="text-xs">Stop</span>
          </button>

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
