<script lang="ts">
  import { platform } from '@tauri-apps/plugin-os';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import * as gym from '$lib/gym';
  import AppText from '$lib/components/gym/AppText.svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut, elasticOut } from 'svelte/easing';

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
  
  // Animated properties 
  const borderOpacity = tweened(0, {
    duration: 400,
    easing: elasticOut
  });
  
  let animationFrame: number | null = null;
  let startTime = 0;
  
  // Update animated properties based on recording state
  $effect(() => {
    // Clear any existing animation
    if (animationFrame !== null) {
      cancelAnimationFrame(animationFrame);
      animationFrame = null;
    }
    
    if (recordingState === 'recording') {
      // Initialize the start time
      startTime = Date.now();
      
      // Create a smooth sine wave animation for opacity between 0.3 and 1.0
      const animate = () => {
        const elapsed = Date.now() - startTime;
        // Use sine function to create a wave pattern oscillating between 0.3 and 1.0
        // sin returns -1 to 1, so we adjust to get the range we want
        const opacity = 0.65 + 0.35 * Math.sin(elapsed / 800); // 800ms controls speed of oscillation
        
        borderOpacity.set(opacity);
        animationFrame = requestAnimationFrame(animate);
      };
      
      animationFrame = requestAnimationFrame(animate);
      
    } else if (recordingState === 'stopping') {
      // Solid border for stopping state with full opacity
      borderOpacity.set(1.0);
    } else {
      borderOpacity.set(0);
    }
  });
  
  // Clean up animation on component destruction
  onDestroy(() => {
    if (animationFrame !== null) {
      cancelAnimationFrame(animationFrame);
    }
  });
</script>

<div class="overlay-content">
  {#if recordingState !== 'stopped' && currentQuest}
    <div 
      class="fixed inset-0 pointer-events-none transition-colors duration-300"
      style="border-width: 6px; opacity: {$borderOpacity};"
      class:border-red-500={recordingState === 'recording'}
      class:border-yellow-500={recordingState === 'stopping'}>
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
