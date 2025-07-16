<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { fade, slide } from 'svelte/transition';
  import { Loader, Search } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import type { ForgeTask } from '$lib/types/gym';
  import { getGymCategories, getTasksForGym, createPoolWithApps } from '$lib/api/endpoints/forge';
  import Input from '$lib/components/form/Input.svelte';
  import Button from '$lib/components/form/Button.svelte';
  import CardBg1 from '$lib/assets/card-bg-1.png';
  import CardBg2 from '$lib/assets/card-bg-2.png';
  import CardBg3 from '$lib/assets/card-bg-3.png';
  import CardBg4 from '$lib/assets/card-bg-4.png';
  import GenerateGymModal from "$lib/components/modals/GenerateGymModal.svelte";
  import { walletAddress } from '$lib/stores/wallet';

  // Props
  let tasks: ForgeTask[] = [];
  export let loadingApps: boolean = false;
  export let viewMode: 'preview' = 'preview'; // Changed from 'edit' | 'preview' to only allow 'preview'
  export let isGymBuilder: boolean = false; // Whether this is used in GymBuilder or not
  export let poolId: string | undefined = undefined;
  let showGenerateGymModal = false;
  let currentSkills = '';

  // Filtering state
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();
  let sort: 'lth' | 'htl' = 'htl';
  let search: string = '';
  let showFilters = false;
  let hideAdultTasks = !(localStorage.getItem('gymShowAdult') === 'true');
  // Initialize with localStorage values or defaults
  let minPrice: number | null = parseInt(localStorage.getItem('gymMinPrice') || '0', 10);
  let maxPrice: number | null = parseInt(localStorage.getItem('gymMaxPrice') || '1000', 10);
  let priceRangeMin = 0;
  let priceRangeMax = 500;

  // Background images array - you can replace these with your actual images
  const backgroundImages = [CardBg1, CardBg2, CardBg3, CardBg4];

  onMount(async () => {
    getTasks();
    updateCategories();
  });

  function getReward(task: ForgeTask) {
    return task.rewardLimit || task.app.pool.pricePerDemo || 0;
  }

  function getRandomBackground() {
    return backgroundImages[Math.floor(Math.random() * backgroundImages.length)];
  }

  async function getTasks() {
    loadingApps = true;
    const ts = await getTasksForGym({
      poolId: poolId,
      minReward: minPrice || priceRangeMin,
      maxReward: maxPrice || priceRangeMax,
      query: search,
      categories: Array.from(selectedCategories),
      hideAdult: hideAdultTasks
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
    // Only show tasks that haven't reached their limit
    tasks = ts.filter((t) => !t.uploadLimitReached);
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

  // Save filter settings to localStorage whenever they change
  $: {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('gymMinPrice', minPrice?.toString() || '0');
      localStorage.setItem('gymMaxPrice', maxPrice?.toString() || '500');
      localStorage.setItem('gymShowAdult', (!hideAdultTasks).toString());
    }
  }
</script>

<!-- Available Tasks Section with Dark Theme -->
<div class="min-h-screen px-0 mt-5">
  <div class="max-w-7xl">
    <!-- Header Section -->
    <div class="mb-8">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-4xl font-bold text-white mb-2">The Arena</h1>
          <p class="text-gray-400">Where agents train</p>
        </div>

        <!-- Filter Controls -->
<!--        <div class="flex items-center gap-4">-->
<!--          &lt;!&ndash; Category Filter Buttons &ndash;&gt;-->
<!--          <div class="flex items-center gap-2">-->
<!--            <button-->
<!--              class="px-4 py-2 bg-purple-600 text-white rounded-full font-medium hover:bg-purple-700 transition-colors">-->
<!--              Category-->
<!--              <svg-->
<!--                class="inline-block ml-2 w-4 h-4"-->
<!--                fill="none"-->
<!--                stroke="currentColor"-->
<!--                viewBox="0 0 24 24">-->
<!--                <path-->
<!--                  stroke-linecap="round"-->
<!--                  stroke-linejoin="round"-->
<!--                  stroke-width="2"-->
<!--                  d="M19 9l-7 7-7-7">-->
<!--                </path>-->
<!--              </svg>-->
<!--            </button>-->

<!--            <button-->
<!--              class="px-4 py-2 bg-purple-600 text-white rounded-full font-medium hover:bg-purple-700 transition-colors">-->
<!--              Rewards-->
<!--              <svg-->
<!--                class="inline-block ml-2 w-4 h-4"-->
<!--                fill="none"-->
<!--                stroke="currentColor"-->
<!--                viewBox="0 0 24 24">-->
<!--                <path-->
<!--                  stroke-linecap="round"-->
<!--                  stroke-linejoin="round"-->
<!--                  stroke-width="2"-->
<!--                  d="M19 9l-7 7-7-7">-->
<!--                </path>-->
<!--              </svg>-->
<!--            </button>-->

<!--            <button-->
<!--              class="px-4 py-2 bg-purple-600 text-white rounded-full font-medium hover:bg-purple-700 transition-colors">-->
<!--              Task Type-->
<!--              <svg-->
<!--                class="inline-block ml-2 w-4 h-4"-->
<!--                fill="none"-->
<!--                stroke="currentColor"-->
<!--                viewBox="0 0 24 24">-->
<!--                <path-->
<!--                  stroke-linecap="round"-->
<!--                  stroke-linejoin="round"-->
<!--                  stroke-width="2"-->
<!--                  d="M19 9l-7 7-7-7">-->
<!--                </path>-->
<!--              </svg>-->
<!--            </button>-->

<!--            <button-->
<!--              class="px-4 py-2 bg-purple-600 text-white rounded-full font-medium hover:bg-purple-700 transition-colors">-->
<!--              Rating-->
<!--              <svg-->
<!--                class="inline-block ml-2 w-4 h-4"-->
<!--                fill="none"-->
<!--                stroke="currentColor"-->
<!--                viewBox="0 0 24 24">-->
<!--                <path-->
<!--                  stroke-linecap="round"-->
<!--                  stroke-linejoin="round"-->
<!--                  stroke-width="2"-->
<!--                  d="M19 9l-7 7-7-7">-->
<!--                </path>-->
<!--              </svg>-->
<!--            </button>-->
<!--          </div>-->
<!--        </div>-->
      </div>
    </div>

    {#if showFilters}
      <div transition:slide class="mb-6">
        <div class="bg-gray-800/50 backdrop-blur-sm border border-gray-700/50 rounded-xl p-6">
          <div class="flex flex-col gap-6">
            <!-- Price filter -->
            <div class="flex flex-row gap-8 items-center">
              <div>
                <p class="font-medium text-white mb-2">Price</p>
                <div class="flex flex-row items-center gap-3">
                  <input
                    bind:value={minPrice}
                    max={priceRangeMax + 1}
                    min={priceRangeMin}
                    class="w-20 px-3 py-2 bg-gray-700/50 border border-gray-600/50 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500/50"
                    type="number" />
                  <span class="text-gray-400">to</span>
                  <input
                    bind:value={maxPrice}
                    class="w-20 px-3 py-2 bg-gray-700/50 border border-gray-600/50 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500/50"
                    min={priceRangeMin - 1}
                    max={priceRangeMax}
                    type="number" />
                  <span class="text-gray-400">SOL</span>
                </div>
              </div>
              <div>
                <p class="font-medium text-white mb-2">Sort</p>
                <div class="flex flex-row items-center gap-3">
                  <select
                    bind:value={sort}
                    class="bg-gray-700/50 border border-gray-600/50 text-white rounded-lg focus:ring-purple-500/50 focus:border-purple-500/50 block w-full p-2.5">
                    <option selected value="htl">High to Low</option>
                    <option value="lth">Low to High</option>
                  </select>
                </div>
              </div>
            </div>

            <!-- Categories -->
            <div>
              <p class="font-medium text-white mb-2">Category</p>
              <div class="flex flex-wrap gap-1.5">
                <button
                  class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.size ===
                  0
                    ? 'bg-purple-600 text-white'
                    : 'bg-gray-700/50 text-gray-300 hover:bg-gray-600/50'}"
                  onclick={() => (selectedCategories = new Set())}>
                  All
                </button>
                {#each allCategories as category}
                  <button
                    class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.has(
                      category
                    )
                      ? 'bg-purple-600 text-white'
                      : 'bg-gray-700/50 text-gray-300 hover:bg-gray-600/50'}"
                    onclick={() => toggleCategory(category)}>
                    {category}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Content Filter -->
            <div>
              <p class="font-medium text-white mb-2">Content Filter</p>
              <div class="flex items-center mb-4">
                <input
                  type="checkbox"
                  id="hideAdult"
                  bind:checked={hideAdultTasks}
                  class="rounded border-gray-600 bg-gray-700/50 text-purple-600 focus:ring-purple-500/50" />
                <label for="hideAdult" class="ml-2 text-sm text-gray-300">Hide adult content</label>
              </div>
            </div>

            <div class="flex w-full">
              <button
                class="ml-auto px-6 py-2 bg-purple-600 text-white rounded-lg font-medium hover:bg-purple-700 transition-colors"
                onclick={getTasks}>
                Apply
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}


    <!-- Loading State -->
    {#if loadingApps}
      <div in:fade={{ duration: 100 }} class="flex items-center justify-center h-40">
        <div
          class="animate-spin h-8 w-8 border-4 border-purple-500/30 rounded-full border-t-purple-500">
        </div>
      </div>
    {:else if tasks.length === 0}
      <div in:fade={{ duration: 100 }} class="text-center py-12 text-gray-400">
        <p>No tasks found.</p>
      </div>
    {:else}
      <!-- Tasks Grid -->
      <div in:fade={{ duration: 100 }}>
        <div
          class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 auto-rows-fr">
          <Card
              variant="secondary"
              padding="md"
              className="bg-gray-800/30 backdrop-blur-sm border border-gray-700/50 rounded-2xl overflow-hidden hover:border-purple-500/50 hover:shadow-xl hover:shadow-purple-500/10 transition-all duration-300 h-full flex flex-col">
            <div
                class="flex flex-col h-full items-center justify-center cursor-pointer"
                role="button"
                tabindex="0"
                onkeydown={() => (showGenerateGymModal = true)}
                onclick={() => (showGenerateGymModal = true)}>

              <div
                      class="rounded-full bg-gray-200 w-24 h-24 flex items-center justify-center mb-4 duration-300 transform transition-all">
                <span class="text-5xl text-gray-500 font-light transition-colors duration-300">+</span>
              </div>
              <div
                      class="text-lg font-medium text-gray-600 group-hover:text-gray-800 transition-colors duration-300">
                Create New Dojo
              </div>
              <p
                      class="text-sm text-gray-500 text-center mt-2 group-hover:text-gray-600 transition-colors duration-300">
                Start collecting demonstrations for your AI agent training
              </p>
            </div>
          </Card>
          {#each tasks as task}
            {#if isGymBuilder || !task.uploadLimitReached}
              <a
                href="/app/gym/chat?prompt={encodeURIComponent(
                  task.prompt
                )}&app={encodeURIComponent(
                  JSON.stringify({
                    type: 'website',
                    name: task.app.name,
                    url: `https://${task.app.domain}`,
                    task_id: task._id
                  })
                )}&poolId={task.app.pool._id}"
                class="block group">
                <!-- Task Card -->
                <div
                  class="bg-gray-800/30 backdrop-blur-sm border border-gray-700/50 rounded-2xl overflow-hidden hover:border-purple-500/50 hover:shadow-xl hover:shadow-purple-500/10 transition-all duration-300 h-full flex flex-col">
                  <!-- Card Header with Background -->
                  <div
                    class="relative h-40 flex items-center justify-center overflow-hidden bg-cover bg-center m-2 rounded-2xl"
                    style="background-image: url({getRandomBackground()})">
                    <!-- Logo -->
                    <div class="relative z-10 w-20 h-20 flex items-center justify-center">
                      <img
                        src={getFaviconUrl(task.app.domain)}
                        alt={`${task.app.name} icon`}
                        class="w-16 h-16" />
                    </div>

                    <!-- Top badges -->
                    <div class="absolute top-3 left-3 flex gap-2">
                      {#if isGymBuilder && task.uploadLimitReached}
                        <div
                          class="bg-red-500/90 backdrop-blur-sm text-white px-2 py-1 rounded-full text-xs font-medium">
                          Limit Reached
                        </div>
                      {/if}
                    </div>
                  </div>

                  <!-- Card Content -->
                  <div class="p-4 flex-1 flex flex-col">
                    <!-- App Name and SOMNIS count -->
                    <div class="flex items-center justify-between mb-3">
                      <h3 class="text-white text-sm">
                        <span class="text-2xl">{getReward(task)}</span>
                        {task.app.pool.token ? task.app.pool.token.symbol : ''}
                      </h3>
                    </div>

                    <!-- Task Description -->
                    <h4 class="text-white font-medium mb-2 line-clamp-2">
                      {task.prompt.split(' ').slice(0, 6).join(' ')}{task.prompt.split(' ').length >
                      6
                        ? '...'
                        : ''}
                    </h4>

                    <!-- Task Details -->
                    <p class="text-gray-400 text-sm flex-1 line-clamp-3 mb-4">
                      {task.prompt}
                    </p>

                    <!-- Bottom Action -->
                    <div class="mt-auto">
                      <button
                        class="w-full bg-purple-600/80 hover:bg-purple-600 text-white py-2.5 rounded-lg font-medium transition-colors group-hover:bg-purple-500">
                        Start Dojo
                      </button>
                    </div>
                  </div>
                </div>
              </a>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<GenerateGymModal
        show={showGenerateGymModal}
        skills={currentSkills}
        on:change={(e) => (currentSkills = e.detail.skills)}
        on:close={() => (showGenerateGymModal = false)}
        on:save={async (event) => {
    try {
      if (!$walletAddress) return;
     const TOKEN_DATA = {
        OMNIS: 'G6iRK8kN67HJFrPA1CDA5KZaPJMiBu3bqdd9vdKBpump', // VIRAL token
        SOL: 'So11111111111111111111111111111111111111112' // SOL token
      };

      const generatedResponse = event.detail.generatedResponse;
      if (generatedResponse?.content) {
        const { name, apps } = generatedResponse.content;

        // Create pool with the generated apps and name
        await createPoolWithApps({
          name,
          skills: currentSkills,
          token: {
            type: 'SOL',
            symbol: 'SOL',
            address: TOKEN_DATA.SOL
          },
          apps,
          ownerAddress: $walletAddress
        });

        currentSkills = '';
        showGenerateGymModal = false;
        // loadPools();
      }
    } catch (err) {
      err = err instanceof Error ? err.message : 'Failed to create AI agent gym';
    }
  }} />