<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, Database, ChevronRight, Wand2, Folder } from 'lucide-svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import CreateDatasetModal from '$lib/components/CreateDatasetModal.svelte';
  import DatasetDetailsModal from '$lib/components/DatasetDetailsModal.svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  interface Dataset {
    id: string;
    name: string;
    path: string;
    size: string;
    numRecordings: number;
    category: string;
    dateCreated: string;
  }
  
  let showCreateModal = $state(false);
  let showDetailsModal = $state(false);
  let selectedDataset = $state<Dataset | null>(null);
  
  let allDatasets = $state<Dataset[]>([]);
  let searchQuery = $state('');
  let filteredDatasets = $state<Dataset[]>([]);
  let allCategories = $state<string[]>([]);
  let selectedCategories = $state(new Set<string>());
  let showFilters = $state(false);
  let loading = $state(true);
  let error = $state('');
  
  function viewDatasetDetails(dataset: Dataset) {
    selectedDataset = dataset;
    showDetailsModal = true;
  }

  function toggleCategory(category: string) {
    if (selectedCategories.has(category)) {
      selectedCategories.delete(category);
    } else {
      selectedCategories.add(category);
    }
    filterDatasets();
  }

  function filterDatasets() {
    // Filter by search query and categories
    filteredDatasets = allDatasets.filter(dataset => {
      const matchesQuery = searchQuery === '' || 
        dataset.name.toLowerCase().includes(searchQuery.toLowerCase());
      
      const matchesCategories = selectedCategories.size === 0 || 
        selectedCategories.has(dataset.category);
      
      return matchesQuery && matchesCategories;
    });
  }
  
  // Utility to calculate sizes
  function formatSize(size: number): string {
    if (size < 1024) return `${size} B`;
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`;
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  
  async function scanDownloadsFolder() {
    loading = true;
    error = '';
    allDatasets = [];
    
    try {
      // Check if the engine is available
      const engineStatus = await invoke<any>('get_engine_status').catch(() => null);
      
      if (engineStatus) {
        // Get download path from settings
        const settings = await invoke<any>('get_settings').catch(() => ({ engines: { downloads_path: '~/.viralmind/downloads' } }));
        const baseDownloadPath = settings?.engines?.downloads_path || '~/.viralmind/downloads';
        
        // Get total size and count of recording folders
        const sizeCmd = `du -sh ${baseDownloadPath} | cut -f1`;
        const sizeJobId = await invoke<string>('run_command', { command: sizeCmd });
        
        // Command to count directories (excluding . and ..)
        const countCmd = `find ${baseDownloadPath} -maxdepth 1 -type d | grep -v "^downloads$" | wc -l`;
        const countJobId = await invoke<string>('run_command', { command: countCmd });
        
        // Wait for size command completion
        let sizeOutput = '';
        while (true) {
          const job = await invoke<any>('get_job', { jobId: sizeJobId });
          if (job.status !== 'Running') {
            sizeOutput = job.output || '0';
            break;
          }
          await new Promise(resolve => setTimeout(resolve, 300));
        }
        
        // Wait for count command completion
        let countOutput = '';
        while (true) {
          const job = await invoke<any>('get_job', { jobId: countJobId });
          if (job.status !== 'Running') {
            countOutput = job.output || '0';
            break;
          }
          await new Promise(resolve => setTimeout(resolve, 300));
        }
        
        // Get folder count and total size
        const folderCount = parseInt(countOutput.trim()) || 0;
        const totalSize = sizeOutput.trim() || '0';
        
        // Add the single "Downloads" dataset
        allDatasets.push({
          id: crypto.randomUUID(),
          name: "Downloads",
          path: baseDownloadPath,
          size: totalSize,
          numRecordings: folderCount,
          category: 'Raw Demonstrations',
          dateCreated: new Date().toISOString().split('T')[0]
        });
      } else {
        // If no engine, show message
        error = 'No engine connected. Connect an engine to list downloaded datasets.';
      }
      
      // Update categories
      allCategories = [...new Set(allDatasets.map(dataset => dataset.category))].sort();
      filterDatasets();
      
    } catch (err) {
      console.error('Failed to scan downloads folder:', err);
      error = err instanceof Error ? err.message : 'Failed to scan downloads folder';
    } finally {
      loading = false;
    }
  }
  
  function handleCreateDataset(data: { name: string; plugin: string }) {
    // Create dataset folder structure
    // In a real implementation, this would call an API or set up recording
    showCreateModal = false;
    scanDownloadsFolder(); // Refresh to look for new folders
  }

  // Re-filter when search query changes
  $effect(() => {
    searchQuery;
    filterDatasets();
  });

  // Re-filter when categories change
  $effect(() => {
    selectedCategories;
    filterDatasets();
  });

  onMount(() => {
    scanDownloadsFolder();
  });
</script>

<div class="h-full space-y-6 p-4">
  <div class="mb-6">
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-2xl font-bold">Datasets</h2>
    </div>
    <p class="text-gray-400 mb-3">
      Datasets make up the skills and knowledge base for your AI. Here you can generate those datasets from your recordings or view existing ones.
    </p>
    <!-- <p class="text-gray-400">
      Don't have any data yet? Try <a href="/app/forge" class="text-secondary-400 hover:underline">using the Forge</a> to collect demonstrations!
    </p> -->
  </div>

  <!-- Dataset Cards Grid -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
    <!-- Generate New Dataset Card -->
    <Card variant="secondary" padding="md" className="h-full flex flex-col border-2 border-dashed border-gray-300 bg-gray-50 hover:bg-gray-100 hover:border-secondary-300 transition-all duration-300">
          <div
            class="flex flex-col h-full items-center justify-center cursor-pointer"
            role="button"
            tabindex="0"
            onkeydown={() => (showCreateModal = true)}
            onclick={() => (showCreateModal = true)}>
            <div class="rounded-full bg-gray-200 w-24 h-24 flex items-center justify-center mb-4 transition-colors duration-300 transform transition-transform">
              <Wand2 size={32} class="text-gray-500 transition-colors duration-300" />
            </div>
            <div class="text-lg font-medium text-gray-600 group-hover:text-gray-800 transition-colors duration-300">Generate Training Dataset</div>
        <p class="text-sm text-gray-500 text-center mt-2 group-hover:text-gray-600 transition-colors duration-300">
          Process recordings into training data using AI
        </p>
      </div>
    </Card>
    
    <!-- Dataset Cards -->
    {#each filteredDatasets as dataset}
      <Card variant="secondary" padding="md" className="h-full flex flex-col hover:bg-white hover:shadow-lg! transition-shadow duration-300">
        <div
          class="flex flex-col h-full cursor-pointer transition-opacity duration-300"
          role="button"
          tabindex="0"
          onkeydown={() => viewDatasetDetails(dataset)}
          onclick={() => viewDatasetDetails(dataset)}>
          <!-- Dataset Name -->
          <div class="font-title text-lg mb-2 truncate">{dataset.name}</div>
          
          <!-- Category Badge -->
          <div class="flex items-center gap-1 mb-3">
            <div class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-purple-500/10 text-purple-500">
              <Database size={12} />
              {dataset.category}
            </div>
          </div>
          
          <!-- Stats -->
          <div class="mt-auto">
            <div class="flex items-center justify-between mt-2">
              <div class="text-sm font-medium">
                {dataset.numRecordings.toLocaleString()} <span class="font-medium">Samples</span>
              </div>
              <div class="text-sm text-gray-500">Size: {dataset.size}</div>
            </div>
            
            <div class="flex justify-end mt-2">
              <button 
                class="text-secondary-500 text-sm flex items-center gap-1 hover:text-blue-600 focus:outline-none"
                onclick={(e) => {
                  e.stopPropagation();
                  viewDatasetDetails(dataset);
                }}
              >
                View Details
                <div class="text-gray-400">
                  <ChevronRight size={16} />
                </div>
              </button>
            </div>
          </div>
        </div>
      </Card>
    {/each}
  </div>
  
  <!-- Empty State -->
  {#if filteredDatasets.length === 0}
    <div class="flex flex-col items-center justify-center px-12 text-center">
      <div class="w-16 h-16 mb-4 text-gray-300">
        <Database size={64} />
      </div>
      <h3 class="text-xl font-bold text-gray-800 mb-2">No datasets found</h3>
      <p class="text-gray-500 max-w-md">
        You don't have any datasets yet. Try <a href="/app/forge" class="text-secondary-400 hover:underline">using the Forge</a> to collect some demonstrations first, or generate a dataset from existing recordings.
      </p>
    </div>
  {/if}
</div>

<!-- Create Dataset Modal -->
<CreateDatasetModal 
  show={showCreateModal}
  onClose={() => (showCreateModal = false)}
  onCreate={handleCreateDataset}
/>

<DatasetDetailsModal
  show={showDetailsModal}
  onClose={() => (showDetailsModal = false)}
  dataset={selectedDataset}
/>

<!-- Loading overlay -->
{#if loading}
  <div class="fixed inset-0 bg-black/30 flex items-center justify-center z-30">
    <div class="bg-gray-800 rounded-lg p-6 text-white flex flex-col items-center">
      <div class="animate-spin text-2xl mb-3">⚙️</div>
      <div>Scanning downloaded datasets...</div>
    </div>
  </div>
{/if}
