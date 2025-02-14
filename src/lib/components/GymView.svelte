<script lang="ts">
  import Card from './Card.svelte';
  import { onMount } from 'svelte';
  import { getApps } from '$lib/api/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import GymHeader from './gym/GymHeader.svelte';
  let apps: ForgeApp[] = [];
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();

  onMount(async () => {
    try {
      apps = await getApps();
      // Get unique categories across all apps
      allCategories = [...new Set(apps.flatMap(app => app.categories))].sort();
    } catch (error) {
      console.error('Failed to fetch apps:', error);
    }
  });

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

  $: filteredApps = selectedCategories.size === 0 
    ? apps 
    : apps.filter(app => app.categories.some(cat => selectedCategories.has(cat)));
</script>

<div class="h-full p-2 sm:p-4 max-w-7xl mx-auto overflow-x-hidden">
  <GymHeader title="Train AI and Earn" />

  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4 mb-4 sm:mb-6">
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">1,234 VIRAL</div>
        <div class="text-sm text-gray-500 mb-4">+ 5,678 in other tokens</div>
        <a href="/app/gym/skills" class="inline-block bg-[#bc59fa] text-white px-4 py-2 rounded-lg text-center hover:bg-opacity-90 transition-colors">
          View Skill Tree
        </a>
      </div>
    </Card>
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">567 VIRAL</div>
        <div class="text-sm text-gray-500 mb-4">Unclaimed Rewards</div>
        <a href="/app/gym/history" class="inline-block bg-[#bc59fa] text-white px-4 py-2 rounded-lg text-center hover:bg-opacity-90 transition-colors">
          View History
        </a>
      </div>
    </Card>
  </div>

  <div class="mb-6">
    <div class="flex flex-wrap gap-2">
      <button 
        class="px-4 py-1.5 rounded-full text-sm font-medium transition-colors {selectedCategories.size === 0 ? 'bg-[#bc59fa] text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
        onclick={() => selectedCategories = new Set()}
      >
        All
      </button>
      {#each allCategories as category}
        <button 
          class="px-4 py-1.5 rounded-full text-sm font-medium transition-colors {selectedCategories.has(category) ? 'bg-[#bc59fa] text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
          onclick={() => toggleCategory(category)}
        >
          {category}
        </button>
      {/each}
    </div>
  </div>

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 sm:gap-4 auto-rows-fr w-full">
    {#each filteredApps as app}
      {#each app.tasks as task}
        <a href="/app/gym/chat?prompt={encodeURIComponent(task.prompt)}&app={encodeURIComponent(JSON.stringify({
          type: 'website',
          name: app.name,
          url: `https://${app.domain}`
        }))}" class="block">
          <Card padding="lg" className="relative h-full hover:border-[#bc59fa] border-4 border-[#f7edfd] hover:bg-gray-50 transition-colors">
            <div class="text-md text-neutral-800 font-medium break-words mb-8">
              {task.prompt}
            </div>
            <div class="absolute bottom-2 left-2 right-2 flex items-center gap-2">
              <img 
                src={getFaviconUrl(app.domain)} 
                alt={`${app.name} icon`}
                class="w-6 h-6"
              />
              <span class="text-sm text-gray-500 truncate">{app.name}</span>
            </div>
          </Card>
        </a>
      {/each}
    {/each}
  </div>
</div>
