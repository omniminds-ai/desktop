<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { Target, Award } from 'lucide-svelte';

  type RecordingState = 'recording' | 'stopping' | 'stopped';
  
  interface RecordingStatus {
    state: RecordingState;
  }

  let recordingState = $state<RecordingState>('stopped');
  let recordingTime = $state(0);
  let timer: number;
  
  let currentQuest = $state({
    title: "Create a New Spreadsheet",
    description: "Open Excel or Google Sheets and create a new spreadsheet with at least 3 columns and 5 rows of data",
    objectives: [
      { text: "Open spreadsheet application", completed: false },
      { text: "Create new document", completed: false },
      { text: "Add required data", completed: false }
    ],
    reward: 10
  });

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
    currentQuest.objectives.forEach(obj => obj.completed = false);
  }

  onMount(() => {
    let unlisten: () => void;
    
    listen<RecordingStatus>('recording-status', (event) => {
      recordingState = event.payload.state;
      
      if (event.payload.state === 'recording') {
        startTimer();
        // Simulate completing objectives over time
        setTimeout(() => {
          currentQuest.objectives[0].completed = true;
          completedObjectives++;
        }, 2000);
        setTimeout(() => {
          currentQuest.objectives[1].completed = true;
          completedObjectives++;
        }, 4000);
      } else if (event.payload.state === 'stopped') {
        stopTimer();
        resetQuest();
      }
    }).then(unlistenFn => {
      unlisten = unlistenFn;
    });

    return () => {
      unlisten?.();
    };
  });

  onDestroy(() => {
    stopTimer();
  });
</script>

<div class="overlay-content">
  {#if recordingState !== 'stopped'}
    <div class="fixed top-4 right-4 w-96 bg-black/80 rounded-lg border border-purple-500/50 p-4 text-white shadow-lg backdrop-blur-sm">
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <span class="px-2 py-1 text-xs font-semibold bg-purple-600 rounded-full">Active Quest</span>
          <span class="px-2 py-1 text-xs font-semibold bg-green-600 rounded-full">{currentQuest.reward} $VIRAL</span>
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
      
      <p class="text-sm text-gray-300 mb-4">
        {currentQuest.description}
      </p>

      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <Target class="text-purple-400" size={16} />
          <span class="text-sm">Progress: {completedObjectives}/3 objectives complete</span>
        </div>
        
        <div class="w-full bg-purple-950 rounded-full h-2">
          <div 
            class="bg-purple-600 h-2 rounded-full transition-all duration-300" 
            style="width: {(completedObjectives / 3) * 100}%"
          />
        </div>
        
        <div class="space-y-2">
          {#each currentQuest.objectives as objective}
            <div class="flex items-center gap-2" class:text-green-400={objective.completed} class:text-gray-400={!objective.completed}>
              <Award size={14} />
              <span class="text-sm">{objective.text} {objective.completed ? '✓' : ''}</span>
            </div>
          {/each}
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
