<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Upload, X, Check, AlertCircle } from 'lucide-svelte';
  import { uploadQueue, removeFromQueue, cleanupIntervals } from '$lib/uploadManager';

  // Clean up intervals when component is destroyed
  onDestroy(() => {
    cleanupIntervals();
  });

  // Get the status icon based on status
  function getStatusIcon(status: string) {
    switch (status) {
      case 'completed':
        return Check;
      case 'failed':
        return AlertCircle;
      default:
        return Upload;
    }
  }

  // Get the status color based on status
  function getStatusColor(status: string) {
    switch (status) {
      case 'completed':
        return 'text-green-500';
      case 'failed':
        return 'text-red-500';
      case 'processing':
        return 'text-yellow-500';
      default:
        return 'text-secondary-300';
    }
  }

  // Get a human-readable status message
  function getStatusMessage(item: { status: string; error?: string }) {
    if (item.status === 'failed' && item.error) {
      return `Failed: ${item.error}`;
    }

    switch (item.status) {
      case 'queued':
        return 'Queued for upload';
      case 'uploading':
        return 'Uploading...';
      case 'processing':
        return 'Processing...';
      case 'completed':
        return 'Upload completed';
      case 'failed':
        return 'Upload failed';
      default:
        return 'Unknown status';
    }
  }

  $: queue = $uploadQueue;
  $: queueItems = Object.entries(queue).map(([id, item]) => ({ id, ...item }));
</script>

{#if Object.keys(queue).length > 0}
  <div class="bg-gray-800 rounded-lg shadow-lg">
    <div class="bg-gray-700 px-4 py-2 rounded-t-lg flex justify-between items-center">
      <div class="text-white font-medium flex gap-2">
        <span class="text-sm">Upload Manager</span>
      </div>
    </div>

    <div class="max-h-60 overflow-y-auto">
      {#each queueItems as item}
        {@const StatusIcon = getStatusIcon(item.status)}
        <div class="px-4 py-3 border-b border-gray-700 last:border-0">
          <div class="flex justify-between items-start mb-1">
            <div class="flex items-center gap-2">
              <StatusIcon size={16} class={getStatusColor(item.status)} />
              <span class="text-white text-sm truncate max-w-[120px]">{item.name}</span>
            </div>
            <button
              class="text-gray-400 hover:text-white self-center"
              on:click={() => removeFromQueue(item.id)}>
              <X size={14} />
            </button>
          </div>

          <div class="text-xs text-gray-400 mb-1">{getStatusMessage(item)}</div>

          {#if item.progress !== undefined}
            <div class="w-full bg-gray-700 rounded-full h-1.5 mb-1">
              <div
                class="h-1.5 rounded-full {item.status === 'completed'
                  ? 'bg-green-500'
                  : 'bg-secondary-300'}"
                style="width: {item.progress}%">
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}
