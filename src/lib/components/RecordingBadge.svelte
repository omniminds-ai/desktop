<script lang="ts">
import { formatTime } from '../utils';

export let state: 'recording' | 'stopping' | 'stopped' = 'stopped';
export let time = 0;
</script>

<div class="flex items-center gap-2">
  <span class="px-2 py-1 text-xs font-semibold bg-red-600 rounded-full animate-pulse">
    {#if state === 'recording'}
      Recording
    {:else if state === 'stopping'}
      Saving...
    {/if}
  </span>
  {#if state !== 'stopped'}
    <span class="text-red-400 font-mono">{formatTime(time)}</span>
  {/if}
</div>
