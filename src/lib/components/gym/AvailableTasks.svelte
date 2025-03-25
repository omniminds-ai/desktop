<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { fade, slide } from 'svelte/transition';
  import { Loader, Search } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import type { ForgeTask } from '$lib/types/gym';
  import { getGymCategories, getTasksForGym } from '$lib/api/forge';
  import Input from '../Input.svelte';
  import Button from '../Button.svelte';

  // Props
  let tasks: ForgeTask[] = [];
  export let loadingApps: boolean = false;
  export let viewMode: 'preview' = 'preview'; // Changed from 'edit' | 'preview' to only allow 'preview'
  export let isGymBuilder: boolean = false; // Whether this is used in GymBuilder or not
  export let poolId: string | undefined = undefined;
  // these are unused
  // export let onRefresh: (() => Promise<void>) | null = null;
  // export let onGenerateTasks: (() => void) | null = null;

  // Filtering state
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();
  let sort: 'lth' | 'htl' = 'lth';
  let search: string = '';
  let showFilters = false;
  // Initialize with localStorage values or defaults
  let minPrice: number | null = parseInt(localStorage.getItem('gymMinPrice') || '0', 10);
  let maxPrice: number | null = parseInt(localStorage.getItem('gymMaxPrice') || '500', 10);
  let priceRangeMin = 0;
  let priceRangeMax = 500;

  onMount(async () => {
    getTasks();
    updateCategories();
  });

  function getReward(task: ForgeTask) {
    return task.rewardLimit || task.app.pool_id.pricePerDemo || 0;
  }

  async function getTasks() {
    loadingApps = true;
    const ts = await getTasksForGym({
      poolId: poolId,
      minReward: minPrice || priceRangeMin,
      maxReward: maxPrice || priceRangeMax,
      query: search,
      categories: Array.from(selectedCategories)
    });
    // task rewardLimit is set to individaul or the pool's -- won't be undefined from this function
    switch (sort) {
      case 'htl':
        ts.sort((a, b) => getReward(b) - getReward(a));
        break;
      case 'lth':
        ts.sort((a, b) => getReward(a) - getReward(b));
        break;
      default:
        ts.sort((a, b) => getReward(b) - getReward(a));
        break;
    }
    showFilters = false;
    loadingApps = false;
    // only show tasks that haven't reached their limit
    tasks = ts.filter((t) => t.uploadLimitReached === false);
  }

  async function updateCategories() {
    // Get unique categories across all apps
    const cats = await getGymCategories();
    allCategories = [...new Set(cats)].sort();
  }

  function toggleCategory(category: string) {
    if (selectedCategories.has(category)) {
      selectedCategories.delete(category);
    } else {
      selectedCategories.add(category);
    }
    selectedCategories = selectedCategories; // Trigger reactivity
  }

  function getFaviconUrl(domain: string) {
    return `https://s2.googleusercontent.com/s2/favicons?domain=${domain}&sz=64`;
  }

  // Update price range when apps change
  $: if (tasks.length > 0) {
    const prices = tasks.map((task) => task.rewardLimit!);
    priceRangeMin = 0;
    priceRangeMax = Math.max(500, Math.ceil(Math.max(...prices)));

    // Initialize price range if not set or if default values
    if (minPrice === 0 && maxPrice === 500) {
      // Use persisted values or defaults for the first time
      minPrice = parseInt(localStorage.getItem('gymMinPrice') || '0', 10);
      maxPrice = parseInt(localStorage.getItem('gymMaxPrice') || '500', 10);
    }
  }

  // Save min/max price to localStorage whenever they change
  $: {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('gymMinPrice', minPrice?.toString() || '0');
      localStorage.setItem('gymMaxPrice', maxPrice?.toString() || '500');
    }
  }

  // $: filteredApps = apps.filter((app) => {
  //   const matchesCategories =
  //     selectedCategories.size === 0 || app.categories.some((cat) => selectedCategories.has(cat));
  //   const matchesPrice =
  //     app.pool_id.pricePerDemo >= minPrice && app.pool_id.pricePerDemo <= maxPrice;
  //   return matchesCategories && matchesPrice;
  // });
</script>

<!-- Available Tasks Heading -->
{#if !isGymBuilder || viewMode === 'preview'}
  <div class="flex items-center justify-between mb-2 mt-6 pl-0.5">
    <div class="flex items-center gap-2">
      <h2 class="text-xl font-bold text-gray-800">Available Tasks</h2>
      <div class="bg-secondary-200 text-white px-2 py-0.5 rounded-full text-xs font-medium">
        {tasks.length}
        Available
      </div>
    </div>

    <div class="flex items-center gap-2">
      {#if selectedCategories.size > 0 || (minPrice || 0) > priceRangeMin || (maxPrice || 500) < priceRangeMax}
        <div class="flex items-center gap-1 text-xs text-gray-500">
          <button
            class="text-secondary-500 hover:text-secondary-600 transition-colors hover:underline"
            onclick={() => {
              selectedCategories = new Set();
              minPrice = priceRangeMin;
              maxPrice = priceRangeMax;
              localStorage.removeItem('gymMinPrice');
              localStorage.removeItem('gymMaxPrice');
              getTasks();
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

  <div class="mb-2 pl-0.5 flex gap-2 w-full">
    <div class="flex gap-2 self-start">
      <Input variant="light" class="w-full!" bind:value={search} placeholder="Search tasks">
        {#snippet icon()}
          <Search />
        {/snippet}
      </Input>
      <Button variant="secondary" class="px-3!" onclick={getTasks} behavior="none">Search</Button>
    </div>
  </div>

  {#if showFilters}
    <div transition:slide>
      <Card padding="lg" className="mb-6">
        <div class="flex flex-col gap-6">
          <!-- Price filter -->
          <div class="flex flex-row gap-8 items-center">
            <div>
              <p class="font-medium text-gray-700 mb-2">Price</p>
              <div class="flex flex-row items-center gap-3">
                <Input
                  variant="light"
                  bind:value={minPrice}
                  max={priceRangeMax + 1}
                  min={priceRangeMin}
                  class="w-20!"
                  type="number" />
                <span class="text-gray-500">to</span>
                <Input
                  variant="light"
                  bind:value={maxPrice}
                  class="w-20!"
                  min={priceRangeMin - 1}
                  max={priceRangeMax}
                  type="number" />
                <span class="text-gray-500">$VIRAL</span>
              </div>
            </div>
            <div>
              <p class="font-medium text-gray-700 mb-2">Sort</p>
              <div class="flex flex-row items-center gap-3">
                <select
                  bind:value={sort}
                  class="rounded-lg focus:ring-secondary-300 focus:border-secondary-300 block w-full p-2.5">
                  <option selected value="htl">High to Low</option>
                  <option value="lth">Low to High</option>
                </select>
              </div>
            </div>
          </div>
          <!-- Categories -->
          <div>
            <p class="font-medium text-gray-700 mb-2">Category</p>
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
          <div class="flex w-full">
            <Button class="ml-auto" variant="primary" behavior="none" onclick={getTasks}>
              Apply
            </Button>
          </div>
        </div>
      </Card>
    </div>
  {/if}

  {#if loadingApps}
    <div in:fade={{ duration: 100 }} class="flex items-center justify-center h-40">
      <div
        class="animate-spin h-8 w-8 border-4 border-secondary-300 rounded-full border-t-transparent">
      </div>
    </div>
  {:else if tasks.length === 0}
    <div in:fade={{ duration: 100 }} class="text-center py-12 text-gray-500">
      <p>No tasks found.</p>
    </div>
  {:else}
    <div in:fade={{ duration: 100 }}>
      <div
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 sm:gap-4 auto-rows-fr w-full">
        {#each tasks as task}
          <!-- Skip tasks that have reached their upload limit when in the gym (not in gym builder) -->
          {#if isGymBuilder || !task.uploadLimitReached}
            <a
              href="/app/gym/chat?prompt={encodeURIComponent(task.prompt)}&app={encodeURIComponent(
                JSON.stringify({
                  type: 'website',
                  name: task.app.name,
                  url: `https://${task.app.domain}`,
                  task_id: task._id
                })
              )}&poolId={task.app.pool_id._id}"
              class="block">
              <Card
                padding="none"
                className="relative h-full hover:border-secondary-300 border border-gray-200 hover:shadow-md transition-all overflow-hidden">
                <!-- Task Header with Tag -->
                <div
                  class="bg-gray-50 px-4 py-2 border-b border-gray-200 flex justify-between items-center">
                  <div class="flex items-center gap-2 grow-0">
                    <img
                      src={getFaviconUrl(task.app.domain)}
                      alt={`${task.app.name} icon`}
                      class="w-5 h-5" />
                    <span
                      class="text-sm max-w-72 sm:max-w-48 md:max-w-64 lg:max-w-40 font-medium text-gray-700 truncate">
                      {task.app.name}
                    </span>
                  </div>
                  <div class="grow flex justify-end gap-1">
                    {#if isGymBuilder && task.uploadLimitReached}
                      <div
                        class="bg-red-500 grow-0 w-fit text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1 relative group">
                        <span>Limit Reached</span>
                        <!-- Tooltip with reason -->
                        <div
                          class="absolute bottom-full right-0 mb-2 w-48 bg-gray-900 text-white text-xs rounded p-2 shadow-lg opacity-0 group-hover:opacity-100 transition-opacity z-10 pointer-events-none">
                          {#if task.limitReason}
                            {task.limitReason}
                          {:else if task.uploadLimitReached}
                            {#if task.app.pool_id.uploadLimit.limitType === 'per-day'}
                              Daily gym limit reached ({task.app.gymSubmissions}/{task.app.pool_id
                                .uploadLimit.type})
                            {:else if task.app.pool_id.uploadLimit.limitType === 'total'}
                              Total gym limit reached ({task.app.gymSubmissions}/{task.app.pool_id
                                .uploadLimit.type})
                            {:else}
                              Gym limit reached
                            {/if}
                          {:else}
                            Upload limit reached
                          {/if}
                          {#if task.currentSubmissions !== undefined}
                            <div class="mt-1 pt-1 border-t border-gray-700">
                              Current uploads: {task.currentSubmissions}/{task.uploadLimit}
                            </div>
                          {/if}
                        </div>
                      </div>
                    {/if}
                    <div
                      class="bg-secondary-300 grow-0 w-fit text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1">
                      <Loader size={12} />
                      <span>Task</span>
                    </div>
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
                  <div class="flex items-center gap-2">
                    <div class="text-xs text-black font-black">Click to begin</div>
                    {#if isGymBuilder && task.currentSubmissions !== undefined && task.app.gymLimitType === 'per-task' && task.app.gymLimitValue !== undefined}
                      <div
                        class="text-xs px-1.5 py-0.5 rounded-full {task.currentSubmissions >=
                        task.app.gymLimitValue
                          ? 'bg-red-100 text-red-700'
                          : 'bg-gray-100 text-gray-700'}">
                        {task.currentSubmissions}/{task.app.gymLimitValue}
                      </div>
                    {/if}
                  </div>
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
                    {task.rewardLimit || task.app.pool_id.pricePerDemo || 0} VIRAL
                  </div>
                </div>
              </Card>
            </a>
          {/if}
        {/each}
      </div>
    </div>
  {/if}
{/if}
