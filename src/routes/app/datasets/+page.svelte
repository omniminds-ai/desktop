<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, Database } from 'lucide-svelte';
  import Card from '$lib/components/Card.svelte';
  
  // Mock dataset data
  let allDatasets = [
    {
      id: '1',
      name: 'Desktop Navigation Dataset',
      size: '2.3 GB',
      numRecordings: 145,
      category: 'Desktop',
      dateCreated: '2025-02-14'
    },
    {
      id: '2',
      name: 'Web Interactions Bundle',
      size: '4.7 GB',
      numRecordings: 278,
      category: 'Web',
      dateCreated: '2025-01-22'
    },
    {
      id: '3',
      name: 'Productivity Applications',
      size: '3.1 GB',
      numRecordings: 189,
      category: 'Productivity',
      dateCreated: '2025-02-01'
    },
    {
      id: '4',
      name: 'Gaming User Interface Interactions',
      size: '5.2 GB',
      numRecordings: 203,
      category: 'Gaming',
      dateCreated: '2025-01-05'
    },
    {
      id: '5',
      name: 'Creative Software Demonstrations',
      size: '6.8 GB',
      numRecordings: 311,
      category: 'Creative',
      dateCreated: '2025-02-28'
    },
    {
      id: '6',
      name: 'E-commerce Checkout Flows',
      size: '1.9 GB',
      numRecordings: 102,
      category: 'Web',
      dateCreated: '2025-03-01'
    },
    {
      id: '7',
      name: 'Social Media Interactions',
      size: '3.4 GB',
      numRecordings: 226,
      category: 'Social',
      dateCreated: '2025-01-15'
    },
    {
      id: '8',
      name: 'Office Software Usage Patterns',
      size: '4.2 GB',
      numRecordings: 168,
      category: 'Productivity',
      dateCreated: '2025-02-10'
    }
  ];

  let searchQuery = '';
  let filteredDatasets = [...allDatasets];
  let allCategories = [...new Set(allDatasets.map(dataset => dataset.category))].sort();
  let selectedCategories = new Set();
  let showFilters = false;

  function toggleCategory(category: string) {
    if (selectedCategories.has(category)) {
      selectedCategories.delete(category);
    } else {
      selectedCategories.add(category);
    }
    selectedCategories = selectedCategories; // Trigger reactivity
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

  // Re-filter when search query changes
  $: {
    searchQuery;
    filterDatasets();
  }

  // Re-filter when categories change
  $: {
    selectedCategories;
    filterDatasets();
  }

  onMount(() => {
    // In a real app, this would fetch data from an API
    filterDatasets();
  });
</script>

<div class="h-full max-w-7xl mx-auto overflow-x-hidden">
  <div class="mx-auto mb-8">
    <div class="text-xl font-semibold mb-4">Datasets</div>
    <p class="text-gray-400">
      Browse and search our collection of desktop interaction datasets. These datasets contain recordings 
      that can be used for training AI models to understand desktop navigation and application usage.
    </p>
  </div>

  <!-- Search and Filter Controls -->
  <div class="mb-6">
    <div class="flex items-center gap-3 mb-4">
      <div class="relative flex-grow">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <Search size={16} class="text-gray-400" />
        </div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search datasets..."
          class="block w-full pl-10 pr-4 py-2.5 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-secondary-300 focus:border-transparent"
        />
      </div>
      
      <button
        class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 transition-colors border border-gray-300 rounded-lg hover:bg-gray-50"
        on:click={() => (showFilters = !showFilters)}>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-4 h-4 transition-transform"
          class:rotate-180={showFilters}
          viewBox="0 0 20 20"
          fill="currentColor">
          <path
            fill-rule="evenodd"
            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
            clip-rule="evenodd" />
        </svg>
        Filters
      </button>
    </div>

    {#if showFilters}
      <Card padding="lg" className="mb-6">
        <div>
          <div class="text-sm font-medium text-gray-700 mb-2">Filter by category</div>
          <div class="flex flex-wrap gap-1.5">
            <button
              class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.size === 0
                ? 'bg-secondary-300 text-white'
                : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
              on:click={() => (selectedCategories = new Set())}>
              All
            </button>
            {#each allCategories as category}
              <button
                class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.has(category)
                  ? 'bg-secondary-300 text-white'
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
                on:click={() => toggleCategory(category)}>
                {category}
              </button>
            {/each}
          </div>
        </div>
      </Card>
    {/if}
  </div>

  <!-- Dataset Results Count -->
  <div class="flex items-center gap-2 mb-4">
    <h2 class="text-xl font-bold text-gray-800">Datasets</h2>
    <div class="bg-secondary-200 text-white px-2 py-0.5 rounded-full text-xs font-medium">
      {filteredDatasets.length} Available
    </div>
  </div>

  <!-- Dataset Cards -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-fr">
    {#each filteredDatasets as dataset}
      <Card padding="none" className="relative h-full hover:border-secondary-300 border border-gray-200 hover:shadow-md transition-all overflow-hidden">
        <!-- Dataset Header -->
        <div class="bg-gray-50 px-4 py-2 border-b border-gray-200 flex justify-between items-center">
          <div class="flex items-center gap-2">
            <Database size={16} class="text-secondary-500" />
            <span class="text-sm font-medium text-gray-700 truncate">{dataset.category}</span>
          </div>
          <div class="bg-secondary-300 text-white px-2 py-0.5 rounded-full text-xs font-medium">
            {dataset.numRecordings} Recordings
          </div>
        </div>
        
        <!-- Dataset Content -->
        <div class="p-4 flex flex-col h-28">
          <div class="text-md text-neutral-800 font-medium mb-2">{dataset.name}</div>
          <div class="text-sm text-gray-500 flex-grow">
            Created on {new Date(dataset.dateCreated).toLocaleDateString()}
          </div>
        </div>
        
        <!-- Dataset Footer -->
        <div class="bg-gray-50 px-4 py-2 border-t border-gray-200 flex justify-between items-center">
          <div class="text-xs text-gray-500 font-medium">
            Click to view details
          </div>
          <div class="text-sm font-semibold text-secondary-600">
            {dataset.size}
          </div>
        </div>
      </Card>
    {/each}
  </div>
  
  <!-- Empty State -->
  {#if filteredDatasets.length === 0}
    <div class="flex flex-col items-center justify-center p-12 text-center">
      <div class="w-16 h-16 mb-4 text-gray-300">
        <Database size={64} />
      </div>
      <h3 class="text-xl font-bold text-gray-800 mb-2">No datasets found</h3>
      <p class="text-gray-500 max-w-md">
        No datasets match your current search criteria. Try adjusting your filters or search query.
      </p>
    </div>
  {/if}
</div>
