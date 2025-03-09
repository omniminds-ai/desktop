<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { slide } from 'svelte/transition';
  import PriceRangeSlider from '$lib/components/gym/PriceRangeSlider.svelte';
  import { ListTodo, Loader, Pencil, Check, X, Eye, Settings, Sparkles } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import type { ForgeApp } from '$lib/types/gym';

  // Props
  export let apps: ForgeApp[] = [];
  export let loadingApps: boolean = false;
  export let viewMode: 'preview' = 'preview'; // Changed from 'edit' | 'preview' to only allow 'preview'
  export let isGymBuilder: boolean = false; // Whether this is used in GymBuilder or not
  export let poolId: string | null = null;
  export let onRefresh: (() => Promise<void>) | null = null;
  export let onGenerateTasks: (() => void) | null = null;

  // Internal state
  let editingAppId = '';
  let editingTaskId = '';
  let editingField = '';
  let editValue = '';
  let currentTaskForAppChange: { appIndex: number; taskIndex: number } | null = null;
  let newAppName = '';
  let newAppDomain = '';
  let showNewAppForm = false;
  let rawAppsJson = '';

  // Filtering state
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();
  let showFilters = false;
  // Initialize with localStorage values or defaults
  let minPrice = parseInt(localStorage.getItem('gymMinPrice') || '0', 10);
  let maxPrice = parseInt(localStorage.getItem('gymMaxPrice') || '99', 10);
  let globalMinPrice = 0;
  let globalMaxPrice = 99;

  onMount(() => {
    updateCategories();
  });

  function updateRawJson() {
    rawAppsJson = JSON.stringify(apps, null, 2);
  }

  function updateCategories() {
    // Get unique categories across all apps
    allCategories = [...new Set(apps.flatMap((app) => app.categories))].sort();
  }

  function getIconUrl(domain: string) {
    if (!domain) return '';

    // Handle domains with or without protocol prefix
    const cleanDomain = domain.replace(/^(https?:\/\/)?(www\.)?/, '');
    return `https://www.google.com/s2/favicons?domain=${cleanDomain}&sz=32`;
  }

  function getFaviconUrl(domain: string) {
    return `https://s2.googleusercontent.com/s2/favicons?domain=${domain}&sz=64`;
  }

  function startEditing(appId: string, taskId: string, field: string, value: string) {
    editingAppId = appId;
    editingTaskId = taskId;
    editingField = field;
    editValue = value;
  }

  function cancelEditing() {
    editingAppId = '';
    editingTaskId = '';
    editingField = '';
    editValue = '';
  }

  function saveEditing() {
    const appIndex = apps.findIndex((app) => app.name === editingAppId);
    if (appIndex === -1) return;

    if (editingField === 'domain') {
      apps[appIndex].domain = editValue;
    } else if (editingField === 'name') {
      apps[appIndex].name = editValue;
    } else if (editingField === 'prompt') {
      const taskIndex = apps[appIndex].tasks.findIndex((_, idx) => idx === parseInt(editingTaskId));
      if (taskIndex !== -1) {
        apps[appIndex].tasks[taskIndex].prompt = editValue;
      }
    }

    dispatchChanges();
    cancelEditing();
  }

  function changeTaskApp(newAppIndex: number) {
    if (!currentTaskForAppChange) return;

    const { appIndex, taskIndex } = currentTaskForAppChange;
    const task = { ...apps[appIndex].tasks[taskIndex] };

    // Remove task from current app
    apps[appIndex].tasks = apps[appIndex].tasks.filter((_, idx) => idx !== taskIndex);

    // Add task to new app
    apps[newAppIndex].tasks.push(task);

    dispatchChanges();
    currentTaskForAppChange = null;
  }

  function addNewApp() {
    if (!newAppName) return;

    const newApp: ForgeApp = {
      name: newAppName,
      domain: newAppDomain,
      description: '',
      categories: [],
      tasks: [],
      pool_id:
        apps.length > 0
          ? { ...apps[0].pool_id }
          : {
              _id: '',
              name: '',
              status: '',
              pricePerDemo: 1
            }
    };

    apps = [...apps, newApp];

    // If we were moving a task to this new app
    if (currentTaskForAppChange) {
      changeTaskApp(apps.length - 1);
    }

    dispatchChanges();
    resetNewAppForm();
  }

  function resetNewAppForm() {
    newAppName = '';
    newAppDomain = '';
    showNewAppForm = false;
  }

  function toggleCategory(category: string) {
    if (selectedCategories.has(category)) {
      selectedCategories.delete(category);
    } else {
      selectedCategories.add(category);
    }
    selectedCategories = selectedCategories; // Trigger reactivity
  }

  // Update price range when apps change
  $: if (apps.length > 0) {
    const prices = apps.map((app) => app.pool_id.pricePerDemo);
    globalMinPrice = 0;
    globalMaxPrice = Math.max(99, Math.ceil(Math.max(...prices)));

    // Initialize price range if not set or if default values
    if (minPrice === 0 && maxPrice === 99) {
      // Use persisted values or defaults for the first time
      minPrice = parseInt(localStorage.getItem('gymMinPrice') || '0', 10);
      maxPrice = parseInt(localStorage.getItem('gymMaxPrice') || '99', 10);
    }
  }

  // Save min/max price to localStorage whenever they change
  $: {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('gymMinPrice', minPrice.toString());
      localStorage.setItem('gymMaxPrice', maxPrice.toString());
    }
  }

  $: filteredApps = apps.filter((app) => {
    const matchesCategories =
      selectedCategories.size === 0 || app.categories.some((cat) => selectedCategories.has(cat));
    const matchesPrice =
      app.pool_id.pricePerDemo >= minPrice && app.pool_id.pricePerDemo <= maxPrice;
    return matchesCategories && matchesPrice;
  });

  function dispatchChanges() {
    const event = new CustomEvent('change', {
      detail: {
        apps
      }
    });

    // Dispatch the event to notify parent components
    if (isGymBuilder) {
      updateCategories();
    }

    dispatchEvent(event);
  }
</script>

<!-- Available Tasks Heading -->
{#if !isGymBuilder || viewMode === 'preview'}
  <div class="flex items-center justify-between mb-4 mt-6">
    <div class="flex items-center gap-2">
      <h2 class="text-xl font-bold text-gray-800">Available Tasks</h2>
      <div class="bg-secondary-200 text-white px-2 py-0.5 rounded-full text-xs font-medium">
        {filteredApps.reduce((count, app) => count + app.tasks.length, 0)} Available
      </div>
    </div>

    <div class="flex items-center gap-2">
      {#if selectedCategories.size > 0 || minPrice > globalMinPrice || maxPrice < globalMaxPrice}
        <div class="flex items-center gap-1 text-xs text-gray-500">
          <button
            class="text-secondary-500 hover:text-secondary-600 transition-colors hover:underline"
            onclick={() => {
              selectedCategories = new Set();
              minPrice = globalMinPrice;
              maxPrice = globalMaxPrice;
              localStorage.removeItem('gymMinPrice');
              localStorage.removeItem('gymMaxPrice');
            }}>
            Reset Filters
          </button>
        </div>
      {/if}
      <button
        class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-gray-700 hover:text-gray-900 transition-colors"
        onclick={() => (showFilters = !showFilters)}>
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
  </div>

  {#if showFilters}
    <div transition:slide>
      <Card padding="lg" className="mb-6">
        <div class="flex flex-col gap-6">
          <!-- Price filter -->
          <div>
            <div class="text-sm font-medium text-gray-700 mb-2">Filter by reward</div>
            <PriceRangeSlider
              bind:minPrice
              bind:maxPrice
              globalMin={globalMinPrice}
              globalMax={globalMaxPrice} />
          </div>

          <!-- Categories -->
          <div>
            <div class="text-sm font-medium text-gray-700 mb-2">Filter by category</div>
            <div class="flex flex-wrap gap-1.5">
              <button
                class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.size ===
                0
                  ? 'bg-secondary-300 text-white'
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
                onclick={() => (selectedCategories = new Set())}>
                All
              </button>
              {#each allCategories as category}
                <button
                  class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.has(
                    category
                  )
                    ? 'bg-secondary-300 text-white'
                    : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
                  onclick={() => toggleCategory(category)}>
                  {category}
                </button>
              {/each}
            </div>
          </div>
        </div>
      </Card>
    </div>
  {/if}

  {#if loadingApps}
    <div class="flex items-center justify-center h-40">
      <div
        class="animate-spin h-8 w-8 border-4 border-secondary-500 rounded-full border-t-transparent">
      </div>
    </div>
  {:else if apps.length === 0}
    <div class="text-center py-12 text-gray-500">
      <p>No tasks available.</p>
    </div>
  {:else}
    <div>
      <div
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 sm:gap-4 auto-rows-fr w-full">
        {#each filteredApps as app}
          {#each app.tasks as task}
            <a
              href="/app/gym/chat?prompt={encodeURIComponent(task.prompt)}&app={encodeURIComponent(
                JSON.stringify({
                  type: 'website',
                  name: app.name,
                  url: `https://${app.domain}`
                })
              )}&poolId={app.pool_id._id}"
              class="block">
              <Card
                padding="none"
                className="relative h-full hover:border-secondary-300 border border-gray-200 hover:shadow-md transition-all overflow-hidden">
                <!-- Task Header with Tag -->
                <div
                  class="bg-gray-50 px-4 py-2 border-b border-gray-200 flex justify-between items-center">
                  <div class="flex items-center gap-2">
                    <img src={getFaviconUrl(app.domain)} alt={`${app.name} icon`} class="w-5 h-5" />
                    <span class="text-sm font-medium text-gray-700 truncate">{app.name}</span>
                  </div>
                  <div
                    class="bg-secondary-300 text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1">
                    <Loader size={12} />
                    <span>Task</span>
                  </div>
                </div>

                <!-- Task Content -->
                <div class="p-4 flex flex-col">
                  <div
                    class="text-md text-neutral-800 font-medium break-words overflow-y-auto flex-grow">
                    {task.prompt}
                  </div>
                </div>

                <!-- Task Footer -->
                <div
                  class="bg-gray-50 px-4 py-2 border-t border-gray-200 flex justify-between items-center mt-auto">
                  <div class="text-xs text-black font-black">Click to begin</div>
                  <div class="text-sm font-semibold text-secondary-600 flex items-center gap-1">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="w-3.5 h-3.5"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round">
                      <line x1="12" y1="1" x2="12" y2="23"></line>
                      <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
                    </svg>
                    {app.pool_id.pricePerDemo} VIRAL
                  </div>
                </div>
              </Card>
            </a>
          {/each}
        {/each}
      </div>
    </div>
  {/if}
{/if}
