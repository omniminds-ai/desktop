<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { onMount } from 'svelte';
  import { getAppsForGym, getBalance, listSubmissions } from '$lib/api/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import Button from '$lib/components/Button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { walletAddress } from '$lib/stores/wallet';
  import { page } from '$app/state';

  const poolId = page.url.searchParams.get('poolId');
  const poolName = page.url.searchParams.get('poolName');

  function formatNumber(num: number): string {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  let apps: ForgeApp[] = [];
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();
  let viralBalance = 0;
  let unclaimedRewards = 0;

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

  $: filteredApps =
    selectedCategories.size === 0
      ? apps
      : apps.filter((app) => app.categories.some((cat) => selectedCategories.has(cat)));
</script>

<div class="h-full max-w-7xl mx-auto overflow-x-hidden">
  {#if poolName}
  <div class="mx-auto mb-8">
    <div class="text-xl font-semibold mb-2">Viewing Forge: {poolName}</div>
  </div>
  {:else}
  <div class="mx-auto mb-8">
    <p class="text-gray-400">
      Share your computer skills, earn rewards, and help build the world's largest open-source
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

  <div class="mb-6">
    <div class="flex flex-wrap gap-2">
      <button
        class="px-4 cursor-pointer py-1.5 rounded-full text-sm font-medium transition-colors {selectedCategories.size ===
        0
          ? 'bg-secondary-300 text-white'
          : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
        onclick={() => (selectedCategories = new Set())}>
        All
      </button>
      {#each allCategories as category}
        <button
          class="px-4 cursor-pointer py-1.5 rounded-full text-sm font-medium transition-colors {selectedCategories.has(
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
            <div class="absolute bottom-2 left-2 right-2 flex items-center gap-2">
              <img src={getFaviconUrl(app.domain)} alt={`${app.name} icon`} class="w-6 h-6" />
              <span class="text-sm text-gray-500 truncate">{app.name}</span>
            </div>
          </Card>
        </a>
      {/each}
    {/each}
  </div>
</div>
