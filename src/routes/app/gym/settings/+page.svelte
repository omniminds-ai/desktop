<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Input from '$lib/components/form/Input.svelte';
  import { Plus, X } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { blacklistedApps } from '$lib/stores/blacklist';

  let apps: { name: string; icon?: string }[] = [];
  let currentBlacklist: string[] = [];
  let searchQuery = '';

  blacklistedApps.subscribe((value) => {
    currentBlacklist = value;
  });

  onMount(async () => {
    try {
      const response = await invoke('list_apps', { includeIcons: true });
      apps = response as { name: string; icon?: string }[];
    } catch (error) {
      console.error('Failed to fetch apps:', error);
    }
  });

  $: filteredApps = apps.filter((app) =>
    app.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function addToBlacklist(appName: string) {
    if (!currentBlacklist.includes(appName)) {
      blacklistedApps.set([...currentBlacklist, appName]);
    }
  }

  function removeFromBlacklist(appName: string) {
    blacklistedApps.set(currentBlacklist.filter((app) => app !== appName));
  }
</script>

<div class="h-full mx-auto">
  <div class="">
    <Card padding="lg" className="mb-6">
      <h3 class="text-xl font-title mb-4">App Blacklist</h3>
      <p class="text-gray-700 mb-6">
        Apps in this list will be excluded from recording sessions when uploading.
      </p>

      <div class="space-y-6">
        <!-- Blacklisted Apps -->
        {#if currentBlacklist.length > 0}
          <div class="flex flex-wrap gap-2">
            {#each currentBlacklist as app}
              <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-100 rounded-full">
                <span class="text-sm text-gray-700">{app}</span>
                <button
                  class="text-gray-400 hover:text-gray-600"
                  onclick={() => removeFromBlacklist(app)}>
                  <X class="w-4 h-4" />
                </button>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Search and Add Apps -->
        <div class="space-y-4">
          <Input
            type="text"
            variant="light"
            placeholder="Search apps..."
            bind:value={searchQuery}
            class="w-full" />

          <div class="grid gap-2 h-[30vh] overflow-y-auto">
            {#each filteredApps as app}
              {#if !currentBlacklist.includes(app.name)}
                <button
                  class="flex h-10 items-center justify-between w-full px-4 py-2 text-left hover:cursor-pointer hover:bg-gray-100 duration-75 transition-colors rounded-lg group"
                  onclick={() => addToBlacklist(app.name)}>
                  <div class="flex items-center gap-3">
                    {#if app.icon}
                      <img src={app.icon} alt={app.name} class="w-6 h-6 object-contain" />
                    {/if}
                    <span class="text-gray-700">{app.name}</span>
                  </div>
                  <Plus class="w-4 h-4 text-gray-400 group-hover:text-secondary-300" />
                </button>
              {/if}
            {/each}
          </div>
        </div>
      </div>
    </Card>
  </div>
</div>
