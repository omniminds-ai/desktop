<script lang="ts">
  import { slide } from 'svelte/transition';
  import Card from '$lib/components/Card.svelte';
  import { onMount } from 'svelte';
  import { getAppsForGym, getBalance, listSubmissions } from '$lib/api/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import Button from '$lib/components/Button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { walletAddress } from '$lib/stores/wallet';
  import { page } from '$app/state';
  import PriceRangeSlider from '$lib/components/gym/PriceRangeSlider.svelte';

  const poolId = page.url.searchParams.get('poolId');
  const poolName = page.url.searchParams.get('poolName');

  function formatNumber(num: number): string {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  let apps: ForgeApp[] = [];
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();
  let minPrice = 1;
  let maxPrice = 10;
  let globalMinPrice = 1;
  let globalMaxPrice = 10;
  let viralBalance = 0;
  let unclaimedRewards = 0;
  let showFilters = false;
  async function loadBalance(address: string) {
    try {
      viralBalance = await getBalance(address);
      await listSubmissions(address);
    } catch (error) {
      console.error('Failed to load balance:', error);
    }
  }

  async function loadUnclaimedRewards() {
    try {
      const recordings = await invoke<any[]>('list_recordings');
      let submissions: any[] = [];
      if ($walletAddress) {
        submissions = await listSubmissions($walletAddress);
      }
      
      unclaimedRewards = recordings.reduce((total, recording) => {
        // Only count if recording is completed, has a reward, and hasn't been submitted
        if (recording.status === 'completed' && 
            recording.quest?.reward?.max_reward && 
            !submissions.some(s => s.meta?.id === recording.id)) {
          return total + recording.quest.reward.max_reward;
        }
        return total;
      }, 0);
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }

  onMount(async () => {
    loadUnclaimedRewards();
    if ($walletAddress) {
      loadBalance($walletAddress);
    }
    try {
      apps = await getAppsForGym(poolId || undefined);
      // Get unique categories across all apps
      allCategories = [...new Set(apps.flatMap((app) => app.categories))].sort();
    } catch (error) {
      console.error('Failed to fetch apps:', error);
    }
  });

  // Subscribe to wallet address changes
  $: if ($walletAddress) {
    loadBalance($walletAddress);
  } else {
    // Reset balance and reload unclaimed rewards when wallet disconnected
    viralBalance = 0;
    loadUnclaimedRewards();
  }

  function getFaviconUrl(domain: string) {
    return `https://s2.googleusercontent.com/s2/favicons?domain=${domain}&sz=64`;
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
    const prices = apps.map(app => app.pool_id.pricePerDemo);
    globalMinPrice = 1;
    globalMaxPrice = Math.max(10, Math.ceil(Math.max(...prices)));
    
    // Initialize price range if not set
    if (minPrice === 1 && maxPrice === 10) {
      minPrice = globalMinPrice;
      maxPrice = Math.min(10, globalMaxPrice);
    }
  }

  $: filteredApps = apps.filter(app => {
    const matchesCategories = selectedCategories.size === 0 || 
      app.categories.some(cat => selectedCategories.has(cat));
    const matchesPrice = app.pool_id.pricePerDemo >= minPrice && 
      app.pool_id.pricePerDemo <= maxPrice;
    return matchesCategories && matchesPrice;
  });
</script>

<div class="h-full max-w-7xl mx-auto overflow-x-hidden">
  {#if poolName}
  <div class="mx-auto mb-8">
    <div class="text-xl font-semibold mb-2">Viewing Forge: {poolName}</div>
  </div>
  {:else}
  <div class="mx-auto mb-8">
    <p class="text-gray-400">
      Choose a task, record a demonstration on your desktop, earn rewards. Help build the world's largest open-source
      dataset for training AI assistants.
    </p>
  </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4 mb-4 sm:mb-6">
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">{formatNumber(viralBalance + unclaimedRewards)} VIRAL</div>
        <div class="text-sm text-gray-500 mb-4">+ {formatNumber(unclaimedRewards)} unclaimed</div>
        <Button href="/app/gym/skills" variant="primary" behavior="none">View Skill Tree</Button>
      </div>
    </Card>
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">{formatNumber(unclaimedRewards)} VIRAL</div>
        <div class="text-sm text-gray-500 mb-4">Unclaimed Rewards</div>
        <Button href="/app/gym/history" variant="primary" behavior="none">View History</Button>
      </div>
    </Card>
  </div>

  <!-- Filter toggle -->
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-2">
      <button
        class="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-gray-700 hover:text-gray-900 transition-colors"
        onclick={() => showFilters = !showFilters}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 transition-transform" class:rotate-180={showFilters} viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
        Filters
      </button>
      {#if selectedCategories.size > 0 || minPrice > globalMinPrice || maxPrice < globalMaxPrice}
        <div class="flex items-center gap-1 text-xs text-gray-500">
          <span>•</span>
          <span>{filteredApps.length} task{filteredApps.length === 1 ? '' : 's'}</span>
          <button 
            class="text-secondary-500 hover:text-secondary-600 transition-colors"
            onclick={() => {
              selectedCategories = new Set();
              minPrice = globalMinPrice;
              maxPrice = globalMaxPrice;
            }}
          >
            Clear
          </button>
        </div>
      {/if}
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
              globalMax={globalMaxPrice}
            />
          </div>
          
          <!-- Categories -->
          <div>
            <div class="text-sm font-medium text-gray-700 mb-2">Filter by category</div>
            <div class="flex flex-wrap gap-1.5">
              <button
                class="px-3 cursor-pointer py-1 rounded-full text-xs font-medium transition-colors {selectedCategories.size === 0
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
            padding="lg"
            className="relative h-full hover:border-secondary-300 border-4 border-gray-200 hover:bg-gray-50 transition-colors">
            <div class="text-md text-neutral-800 font-medium break-words mb-8">
              {task.prompt}
            </div>
            <div class="absolute bottom-2 left-2 right-2 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <img src={getFaviconUrl(app.domain)} alt={`${app.name} icon`} class="w-6 h-6" />
                <span class="text-sm text-gray-500 truncate">{app.name}</span>
              </div>
              <div class="text-sm font-medium text-secondary-600">
                {app.pool_id.pricePerDemo} VIRAL
              </div>
            </div>
          </Card>
        </a>
      {/each}
    {/each}
  </div>
</div>
