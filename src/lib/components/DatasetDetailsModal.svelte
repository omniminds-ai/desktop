<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade } from 'svelte/transition';
  import { Database, FileBox, Folder } from 'lucide-svelte';
  import Button from './Button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  const {
    show = false,
    onClose = () => {},
    dataset = null
  } = $props();
  
  // Define an interface for folder items
  interface FolderItem {
    name: string;
    isDir: boolean;
    size: string;
  }
  
  let folderContents = $state<FolderItem[]>([]);
  let loading = $state(false);
  let error = $state('');
  
  $effect(() => {
    if (show && dataset) {
      loadFolderContents();
    }
  });
  
  async function loadFolderContents() {
    if (!dataset || !dataset.path) return;
    
    loading = true;
    error = '';
    folderContents = [];
    
    try {
      // Check if the engine is available
      const engineStatus = await invoke<any>('get_engine_status').catch(() => null);
      
      if (engineStatus) {
        // Use run_command to list folders
        const cmd = `ls -la ${dataset.path}`;
        const jobId = await invoke<string>('run_command', { command: cmd });
        
        // Wait for command completion
        let completed = false;
        let output = '';
        
        while (!completed) {
          const job = await invoke<any>('get_job', { jobId });
          if (job.status !== 'Running') {
            if (job.status === 'Failed') {
              throw new Error(job.error || 'Failed to list folders');
            }
            output = job.output || '';
            completed = true;
          } else {
            // Wait before polling again
            await new Promise(resolve => setTimeout(resolve, 500));
          }
        }
        
        // Parse the output to get folder list
        const lines = output.split('\n').filter(line => line.trim() && !line.startsWith('total'));
        folderContents = lines.map(line => {
          const parts = line.trim().split(/\s+/);
          const name = parts.slice(8).join(' ');
          const isDir = line.startsWith('d');
          return { name, isDir, size: parts[4] || '0' };
        }).filter(f => !f.name.startsWith('.'));
      } else {
        // For display purposes, create some dummy folders
        folderContents = [
          { name: 'No engine connected - cannot list actual folders', isDir: false, size: '' }
        ];
      }
    } catch (err) {
      console.error('Failed to load folder contents:', err);
      error = err instanceof Error ? err.message : 'Failed to load folder contents';
    } finally {
      loading = false;
    }
  }
</script>

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    transition:fade={{ duration: 100 }}>
    <div class="bg-white rounded-lg w-full max-w-2xl">
      <div class="p-4 border-b border-gray-200 flex justify-between items-center">
        <h3 class="text-lg font-bold flex items-center gap-2">
          <Database size={18} class="text-secondary-500" />
          {dataset ? dataset.name : 'Dataset'} Details
        </h3>
        <button 
          class="text-gray-500 hover:text-gray-700"
          onclick={() => onClose()}>
          &times;
        </button>
      </div>
      
      <div class="px-4 py-2 bg-gray-50 border-b border-gray-200">
        {#if dataset}
          <div class="grid grid-cols-2 gap-2 text-sm">
            <div class="flex items-center gap-1">
              <span class="text-gray-500">Size:</span>
              <span class="font-medium">{dataset.size}</span>
            </div>
            <div class="flex items-center gap-1">
              <span class="text-gray-500">Samples:</span>
              <span class="font-medium">{dataset.numRecordings}</span>
            </div>
            <div class="flex items-center gap-1 col-span-2">
              <span class="text-gray-500">Path:</span>
              <span class="font-mono text-xs text-gray-700 bg-gray-100 px-1 py-0.5 rounded">{dataset.path}</span>
            </div>
          </div>
        {/if}
      </div>
      
      {#if error}
        <div class="p-3 m-4 bg-red-50 text-red-500 text-sm rounded-lg border border-red-100">
          {error}
        </div>
      {/if}
      
      <div class="p-2 max-h-[350px] overflow-y-auto">
        <div class="flex items-center justify-between px-2 mb-2">
          <h4 class="text-sm font-medium text-gray-700">Contents</h4>
          <div class="text-xs text-gray-500">{folderContents.length} items</div>
        </div>
        
        {#if loading}
          <div class="text-center text-gray-400 py-8">
            <div class="animate-spin inline-block mr-2">⚙️</div>
            Loading folder contents...
          </div>
        {:else if folderContents.length === 0}
          <div class="text-center text-gray-400 py-8">
            No items found
          </div>
        {:else}
          <ul class="divide-y divide-gray-100">
            {#each folderContents as item}
              <li class="flex items-center gap-2 p-2 hover:bg-gray-50 transition-colors">
                {#if item.isDir}
                  <FileBox size={16} class="text-secondary-400 flex-shrink-0" />
                {:else}
                  <div class="w-[16px] h-[16px] flex-shrink-0"></div>
                {/if}
                <span class="flex-grow text-sm truncate">{item.name}</span>
                {#if item.size}
                  <span class="text-xs text-gray-500 flex-shrink-0">{item.size}</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
      
      <div class="p-4 border-t border-gray-200 flex justify-end">
        <Button
          variant="secondary"
          type="button"
          onclick={() => onClose()}>
          Close
        </Button>
      </div>
    </div>
  </div>
{/if}
