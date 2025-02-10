<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { Target, Award } from 'lucide-svelte';

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
  let completedSubgoals = $state<boolean[]>([]);
  let completedObjectives = $state(0);

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
    completedObjectives = 0;
    completedSubgoals = [];
    currentQuest = null;
  }

  function initializeQuest(quest: Quest) {
    currentQuest = quest;
    completedSubgoals = new Array(quest.subgoals.length).fill(false);
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
        <div class="flex items-center gap-2">
          <span class="px-2 py-1 text-xs font-semibold bg-purple-600 rounded-full">
            Active Quest
          </span>
          <span class="px-2 py-1 text-xs font-semibold bg-green-600 rounded-full">10 $VIRAL</span>
          <span class="px-2 py-1 text-xs font-semibold bg-red-600 rounded-full animate-pulse">
            {#if recordingState === 'recording'}
              Recording
            {:else if recordingState === 'stopping'}
              Saving...
            {/if}
          </span>
        </div>
        <span class="text-purple-400 font-mono">{formatTime(recordingTime)}</span>
      </div>

      <h2 class="text-lg font-semibold mb-2">{currentQuest.title}</h2>

      <p class="text-sm text-gray-300 mb-2">
        {currentQuest.concrete_scenario}
      </p>

      <p class="text-sm text-purple-400 mb-4">
        {currentQuest.objective}
      </p>

      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <Target class="text-purple-400" size={16} />
          <span class="text-sm">
            Progress: {completedObjectives}/{currentQuest.subgoals.length} objectives complete
          </span>
        </div>

        <div class="w-full bg-purple-950 rounded-full h-2">
          <div
            class="bg-purple-600 h-2 rounded-full transition-all duration-300"
            style="width: {(completedObjectives / currentQuest.subgoals.length) * 100}%">
          </div>
        </div>

        <div class="space-y-2">
          {#each currentQuest.subgoals as subgoal, i}
            <div
              class="flex items-center gap-2"
              class:text-green-400={completedSubgoals[i]}
              class:text-gray-400={!completedSubgoals[i]}>
              <Award size={14} />
              <span class="text-sm">{subgoal} {completedSubgoals[i] ? '✓' : ''}</span>
            </div>
          {/each}
        </div>

        {#if currentQuest.relevant_applications.length > 0}
          <div class="flex flex-wrap gap-2 mt-2">
            {#each currentQuest.relevant_applications as app}
              <span class="px-2 py-1 text-xs bg-purple-600/20 rounded-full">{app}</span>
            {/each}
          </div>
        {/if}
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
