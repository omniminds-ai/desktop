<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  type RecordingState = 'recording' | 'stopping' | 'stopped';
  
  interface RecordingStatus {
    state: RecordingState;
  }

  let recordingState = $state<RecordingState>('stopped');

  onMount(() => {
    let unlisten: () => void;
    
    listen<RecordingStatus>('recording-status', (event) => {
      recordingState = event.payload.state;
    }).then(unlistenFn => {
      unlisten = unlistenFn;
    });

    return () => {
      unlisten?.();
    };
  });
</script>

<div class="overlay-content">
  {#if recordingState !== 'stopped'}
    <div class="recording-indicator">
      <div class="dot" class:pulse={recordingState === 'recording'}></div>
      <span>
        {#if recordingState === 'recording'}
          Recording
        {:else if recordingState === 'stopping'}
          Saving Recording...
        {/if}
      </span>
    </div>
  {/if}
</div>

<style>
  .overlay-content {
    width: 100%;
    height: 100%;
    position: relative;
  }

  .recording-indicator {
    position: absolute;
    top: 20px;
    right: 20px;
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 8px 16px;
    border-radius: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .dot {
    width: 8px;
    height: 8px;
    background: #ff4444;
    border-radius: 50%;
  }

  .dot.pulse {
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% { opacity: 1; }
    50% { opacity: 0.5; }
    100% { opacity: 1; }
  }
</style>
