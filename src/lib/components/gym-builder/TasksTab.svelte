<script lang="ts">
  import Card from '../Card.svelte';
  import Button from '../Button.svelte';
  import TextArea from '../TextArea.svelte';
  import { Pencil, Check, X, Eye, Sparkles, DollarSign } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import { onMount } from 'svelte';
  import { getAppsForGym } from '$lib/api/forge';
  import AvailableTasks from '../gym/AvailableTasks.svelte';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
  };
  export let unsavedChanges: boolean;
  export let regenerateTasks: () => void;
  
  let viewMode = "edit"; // edit or preview
  let editingAppId = '';
  let editingTaskId = '';
  let editingField = '';
  let editValue = '';
  let currentTaskForAppChange: { appIndex: number, taskIndex: number } | null = null;
  let newAppName = '';
  let newAppDomain = '';
  let showNewAppForm = false;
  let apps: ForgeApp[] = [];
  let loadingApps = true;
  
  onMount(async () => {
    await loadApps();
  });
  
  async function loadApps() {
    loadingApps = true;
    try {
      apps = await getAppsForGym(pool._id);
    } catch (error) {
      console.error("Failed to load apps:", error);
      apps = [];
    } finally {
      loadingApps = false;
    }
  }

  function updateUnsavedChanges() {
    unsavedChanges = true;
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
    const appIndex = apps.findIndex(app => app.name === editingAppId);
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
    } else if (editingField === 'price') {
      const price = parseFloat(editValue);
      if (!isNaN(price) && price >= 1) {
        pool.pricePerDemo = price;
        pool.unsavedPrice = true;
      }
    }

    unsavedChanges = true;
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
    
    unsavedChanges = true;
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
      pool_id: {
        _id: pool._id,
        name: pool.name,
        status: pool.status,
        pricePerDemo: pool.pricePerDemo || 1
      }
    };
    
    apps.push(newApp);
    
    // If we were moving a task to this new app
    if (currentTaskForAppChange) {
      changeTaskApp(apps.length - 1);
    }
    
    unsavedChanges = true;
    resetNewAppForm();
  }
  
  function resetNewAppForm() {
    newAppName = '';
    newAppDomain = '';
    showNewAppForm = false;
  }

  function getIconUrl(domain: string) {
    if (!domain) return '';
    
    // Handle domains with or without protocol prefix
    const cleanDomain = domain.replace(/^(https?:\/\/)?(www\.)?/, '');
    return `https://www.google.com/s2/favicons?domain=${cleanDomain}&sz=32`;
  }
</script>

<!-- Improved second navbar with consistent styling -->
<div class="flex justify-between items-center border-gray-200 pb-3">
  <div class="flex items-center">
    <h2 class="text-xl font-bold text-gray-800">Tasks</h2>
  </div>
  <div class="flex items-center">
    <div class="bg-gray-100 rounded-lg p-1 flex items-center">
      <button 
        class="px-3 py-1.5 text-sm rounded-md transition-colors flex items-center gap-1
          {viewMode === 'edit' 
            ? 'bg-white text-secondary-600 shadow-sm' 
            : 'text-gray-600 hover:text-gray-800'}"
        onclick={() => viewMode = 'edit'}>
        <Pencil size={15} class="mr-1" />
        Edit
      </button>
      <button
        class="px-3 py-1.5 text-sm rounded-md transition-colors flex items-center gap-1
          {viewMode === 'preview' 
            ? 'bg-white text-secondary-600 shadow-sm' 
            : 'text-gray-600 hover:text-gray-800'}"
        onclick={() => viewMode = 'preview'}>
        <Eye size={15} class="mr-1" />
        Preview
      </button>
    </div>
    <button
      class="ml-2 px-3 py-1.5 text-sm rounded-md bg-secondary-300 text-white hover:bg-secondary-100 transition-colors flex items-center"
      onclick={regenerateTasks}>
      <Sparkles size={15} class="mr-1" />
      Generate Tasks
    </button>
  </div>
</div>

<!-- Scrollable task container -->
<div class="overflow-y-auto px-3" style="max-height: calc(100vh - 220px);">
  <!-- Tasks Tab Content -->
  {#if loadingApps}
    <div class="flex items-center justify-center h-40">
      <div class="animate-spin h-8 w-8 border-4 border-secondary-500 rounded-full border-t-transparent"></div>
    </div>
  {:else if viewMode === 'preview'}
    <!-- Preview Mode using AvailableTasks component -->
    <AvailableTasks
      apps={apps}
      loadingApps={loadingApps}
      viewMode="preview"
      isGymBuilder={true}
      poolId={pool._id}
      onGenerateTasks={regenerateTasks}
    />
  {:else if apps.length === 0}
    <div class="text-center py-12 text-gray-500">
      <p>No apps available yet.</p>
      <p class="mt-2">Click "Regenerate Tasks" to create apps based on your skills.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4">
      {#each apps as app, appIndex}
        <Card padding="md" className="border border-gray-200">
          <div class="relative">
            <!-- App header -->
            <div class="flex justify-between items-start mb-3">
              <div class="flex items-center gap-2">
                <!-- App Icon -->
                {#if editingAppId === app.name && editingField === 'domain'}
                  <div class="flex items-center gap-1">
                    <input 
                      type="text" 
                      class="w-36 px-2 py-1 text-sm border rounded"
                      bind:value={editValue} />
                    <button class="text-green-500 hover:text-green-700" onclick={saveEditing}>
                      <Check size={16} />
                    </button>
                    <button class="text-red-500 hover:text-red-700" onclick={cancelEditing}>
                      <X size={16} />
                    </button>
                  </div>
                {:else}
                  <div class="w-8 h-8 flex items-center justify-center rounded bg-gray-100 cursor-pointer relative group"
                      onclick={() => startEditing(app.name, '', 'domain', app.domain)}>
                    {#if app.domain}
                      <img src={getIconUrl(app.domain)} alt={app.name} class="w-6 h-6" />
                    {:else}
                      <div class="w-6 h-6 bg-gray-300 rounded"></div>
                    {/if}
                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 rounded flex items-center justify-center">
                      <Pencil size={12} class="text-gray-700 opacity-0 group-hover:opacity-100" />
                    </div>
                  </div>
                {/if}

                <!-- App Name -->
                {#if editingAppId === app.name && editingField === 'name'}
                  <div class="flex items-center gap-1">
                    <input 
                      type="text" 
                      class="w-36 px-2 py-1 text-sm border rounded"
                      bind:value={editValue} />
                    <button class="text-green-500 hover:text-green-700" onclick={saveEditing}>
                      <Check size={16} />
                    </button>
                    <button class="text-red-500 hover:text-red-700" onclick={cancelEditing}>
                      <X size={16} />
                    </button>
                  </div>
                {:else}
                  <div class="text-lg font-medium text-gray-800 group relative cursor-pointer"
                      onclick={() => startEditing(app.name, '', 'name', app.name)}>
                    <span>{app.name}</span>
                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/5 rounded flex items-center justify-center">
                      <Pencil size={12} class="text-gray-700 opacity-0 group-hover:opacity-100" />
                    </div>
                  </div>
                {/if}
              </div>

              <!-- Price -->
              {#if editingAppId === app.name && editingField === 'price'}
                <div class="flex items-center gap-1">
                  <input 
                    type="number" 
                    step="0.1"
                    min="1"
                    class="w-20 px-2 py-1 text-sm border rounded"
                    bind:value={editValue} />
                  <span class="text-sm">{pool.token.symbol}</span>
                  <button class="text-green-500 hover:text-green-700" onclick={saveEditing}>
                    <Check size={16} />
                  </button>
                  <button class="text-red-500 hover:text-red-700" onclick={cancelEditing}>
                    <X size={16} />
                  </button>
                </div>
              {:else}
                <div class="text-sm font-bold text-secondary-600 flex items-center gap-1 group relative cursor-pointer"
                    onclick={() => startEditing(app.name, '', 'price', (pool.pricePerDemo || 1).toString())}>
                  <DollarSign size={14} />
                  {pool.pricePerDemo} {pool.token.symbol}
                  <div class="absolute inset-0 bg-black/0 group-hover:bg-black/5 rounded flex items-center justify-center">
                    <Pencil size={12} class="text-gray-700 opacity-0 group-hover:opacity-100" />
                  </div>
                </div>
              {/if}
            </div>

            <!-- App tasks section -->
            <div class="mt-4 pt-3">
              <div class="text-sm font-medium text-gray-700 mb-2">Tasks</div>
              
              {#if app.tasks && app.tasks.length > 0}
                <div class="space-y-2 ml-1">
                  {#each app.tasks as task, taskIndex}
                    {#if editingAppId === app.name && editingTaskId === taskIndex.toString() && editingField === 'prompt'}
                      <div class="mb-2">
                        <TextArea
                          class="w-full text-sm"
                          variant="light"
                          bind:value={editValue}>
                        </TextArea>
                        <div class="flex justify-end mt-2 gap-2">
                          <button class="text-green-500 hover:text-green-700 p-1" onclick={saveEditing}>
                            <Check size={16} />
                          </button>
                          <button class="text-red-500 hover:text-red-700 p-1" onclick={cancelEditing}>
                            <X size={16} />
                          </button>
                        </div>
                      </div>
                    {:else}
                      <div class="relative group cursor-pointer bg-gray-50 p-2 rounded-md hover:bg-gray-100 hover:shadow-sm transition-all duration-300"
                          onclick={() => startEditing(app.name, taskIndex.toString(), 'prompt', task.prompt)}>
                        <p class="text-gray-800 text-sm">{task.prompt}</p>
                        <div class="absolute inset-0 bg-black/0 group-hover:bg-black/5 rounded flex items-center justify-center">
                          <Pencil size={14} class="text-gray-700 opacity-0 group-hover:opacity-100" />
                        </div>
                      </div>
                    {/if}
                  {/each}
                </div>
              {:else}
                <div class="text-sm text-gray-500 italic ml-1">No tasks defined for this app.</div>
              {/if}
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>
