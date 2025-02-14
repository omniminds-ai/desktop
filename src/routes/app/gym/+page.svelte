<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { onMount } from 'svelte';
  import { getAppsForGym } from '$lib/api/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import Button from '$lib/components/Button.svelte';
  let apps: ForgeApp[] = [];
  let allCategories: string[] = [];
  let selectedCategories: Set<string> = new Set();

  onMount(async () => {
    try {
      apps = await getAppsForGym();
      // Get unique categories across all apps
      allCategories = [...new Set(apps.flatMap((app) => app.categories))].sort();
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

  $: filteredApps =
    selectedCategories.size === 0
      ? apps
      : apps.filter((app) => app.categories.some((cat) => selectedCategories.has(cat)));
</script>

<div class="h-full max-w-7xl mx-auto overflow-x-hidden">
  <div class="mx-auto mb-8">
    <p class="text-gray-400">
      Share your computer skills, earn rewards, and help build the world's largest open-source
      dataset for training AI assistants.
    </p>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4 mb-4 sm:mb-6">
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">1,234 VIRAL</div>
        <div class="text-sm text-gray-500 mb-4">+ 5,678 in other tokens</div>
        <Button href="/app/gym/skills" variant="primary" behavior="none">View Skill Tree</Button>
      </div>
    </Card>
    <Card padding="lg" className="relative">
      <div class="flex flex-col">
        <div class="text-2xl font-semibold mb-1">567 VIRAL</div>
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
          )}"
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
