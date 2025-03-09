<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade } from 'svelte/transition';
  import Input from './Input.svelte';
  import Button from './Button.svelte';
  import { Play, Database } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  const {
    show = false,
    onClose = () => {},
    onCreate = (data: { name: string; plugin: string }) => {}
  } = $props();
  
  // Dataset and plugin selection
  let selectedDatasetId = $state('');
  let selectedPlugin = $state('openai-sft');
  let isCreating = $state(false);
  let error = $state('');
  let availableDatasets = $state<{id: string; name: string; path: string}[]>([]);
  
  // The only plugin available for now
  const plugins: {id: string; name: string; path: string}[] = [
    // { id: 'openai-sft', name: 'OpenAI SFT Generator', cmd: './pipeline {dataset}' }
  ];
  
  async function loadAvailableDatasets() {
    try {
      // In a real implementation, this would load datasets from the backend
      // For now, we'll hard-code a simple example
      const settings = await invoke<any>('get_settings').catch(() => ({ engines: { downloads_path: '~/.viralmind/downloads' } }));
      const baseDownloadPath = settings?.engines?.downloads_path || '~/.viralmind/downloads';
      
      availableDatasets = [
        { id: 'downloads', name: 'Downloads', path: baseDownloadPath }
      ];
      
      // Select the first dataset by default if none is selected
      if (!selectedDatasetId && availableDatasets.length > 0) {
        selectedDatasetId = availableDatasets[0].id;
      }
    } catch (err) {
      console.error('Failed to load datasets:', err);
      error = 'Failed to load available datasets';
    }
  }
  
  function handleSubmit() {
    if (!selectedDatasetId) {
      error = 'Please select an input dataset';
      return;
    }
    
    if (!selectedPlugin) {
      error = 'Please select a generator plugin';
      return;
    }
    
    isCreating = true;
    error = '';
    
    try {
      // Get selected dataset path
      const dataset = availableDatasets.find(d => d.id === selectedDatasetId);
      
      if (!dataset) {
        throw new Error('Selected dataset not found');
      }
      
      onCreate({ 
        name: `Generated from ${dataset.name}`, 
        plugin: selectedPlugin 
      });
      resetForm();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to start generation job';
    } finally {
      isCreating = false;
    }
  }
  
  function handleClose() {
    resetForm();
    onClose();
  }
  
  function resetForm() {
    // Keep dataset selection, but clear errors
    error = '';
  }
  
  // When the modal is shown, initialize the form and load datasets
  $effect(() => {
    if (show) {
      resetForm();
      loadAvailableDatasets();
    }
  });
</script>

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    transition:fade={{ duration: 100 }}>
    <div class="bg-white rounded-lg w-full max-w-lg shadow-xl">
      <div class="p-4 border-b border-gray-200 flex justify-between items-center">
        <h3 class="text-lg font-bold">Generate Dataset</h3>
        <button 
          class="text-gray-500 hover:text-gray-700"
          onclick={handleClose}>
          &times;
        </button>
      </div>

      <div class="p-4">
        <p class="text-sm text-gray-600 mb-4">
          Generate a new dataset from existing recordings using AI processing.
        </p>
        
        {#if error}
          <div class="p-3 mb-4 bg-red-50 text-red-500 text-sm rounded-lg border border-red-100">
            {error}
          </div>
        {/if}
        
        <form onsubmit={(e) => e.preventDefault()} class="space-y-4">
          <div>
            <label for="dataset-selector" class="block text-sm font-medium text-gray-700 mb-1">
              Input Dataset
            </label>
            <div class="relative">
              <select
                id="dataset-selector"
                bind:value={selectedDatasetId}
                class="w-full p-2.5 bg-white rounded-lg text-gray-700 border border-gray-300 focus:border-secondary-300 focus:outline-none focus:ring-1 focus:ring-secondary-300"
              >
                <option value="" disabled>Select a dataset</option>
                {#each availableDatasets as dataset}
                  <option value={dataset.id}>
                    {dataset.name} ({dataset.path})
                  </option>
                {/each}
              </select>
              <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <Database size={16} class="text-gray-400" />
              </div>
            </div>
            <p class="mt-1 text-xs text-gray-500">Select the dataset to use as input for generation.</p>
          </div>
          
          <div>
            <label for="plugin-selector" class="block text-sm font-medium text-gray-700 mb-1">
              Generator Plugin
            </label>
            <div class="relative">
              <select
                id="plugin-selector"
                bind:value={selectedPlugin}
                class="w-full p-2.5 bg-white rounded-lg text-gray-700 border border-gray-300 focus:border-secondary-300 focus:outline-none focus:ring-1 focus:ring-secondary-300"
              >
                {#each plugins as plugin}
                  <option value={plugin.id}>{plugin.name}</option>
                {/each}
              </select>
            </div>
            <p class="mt-1 text-xs text-gray-500">AI generator that will process the dataset.</p>
          </div>
        </form>
      </div>
      
      <div class="p-4 border-t border-gray-200 flex justify-between">
        <Button
          variant="secondary"
          type="button"
          onclick={handleClose}>
          Cancel
        </Button>
        <Button
          type="button"
          variant="primary"
          onclick={handleSubmit}
          disabled={true/*isCreating || !selectedDatasetId || !selectedPlugin*/}
          class="flex! items-center gap-2">
          <Play size={16} />
          {isCreating ? 'Starting...' : 'Start Generation Job'}
        </Button>
      </div>
    </div>
  </div>
{/if}
