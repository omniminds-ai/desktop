<script lang="ts">
  import Card from '../Card.svelte';
  import Button from '../Button.svelte';
  import TextArea from '../TextArea.svelte';
  import { Pencil, Check, X, Eye, Sparkles, DollarSign, Train, Plus, Trash2, Save, RotateCcw } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import { onMount } from 'svelte';
  
  // Function to save changes to the backend
  export let handleSaveChanges: () => Promise<void>;
  
  // Wrap the original handleSaveChanges to update originalApps after saving
  async function saveChanges() {
    await handleSaveChanges();
    // Update originalApps after saving
    originalApps = JSON.parse(JSON.stringify(apps));
    pool.unsavedApps = false;
  }
  
  // Store original apps for reset functionality
  let originalApps: ForgeApp[] = [];
  import { getAppsForGym } from '$lib/api/forge';
  import AvailableTasks from '../gym/AvailableTasks.svelte';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
    unsavedApps?: boolean;
  };
  export let unsavedChanges: boolean;
  export let regenerateTasks: () => void;

  let viewMode = 'edit'; // edit or preview
  let editingAppId = '';
  let editingTaskId = '';
  let editingField = '';
  let editValue = '';
  let currentTaskForAppChange: { appIndex: number; taskIndex: number } | null = null;
  let newAppName = '';
  let newAppDomain = '';
  let showNewAppForm = false;
  // Make apps public so it can be accessed by the parent component
  export let apps: ForgeApp[] = [];
  let loadingApps = true;

  onMount(async () => {
    await loadApps();
    // Store a deep copy of the original apps
    originalApps = JSON.parse(JSON.stringify(apps));
  });
  
  // Function to reset changes
  function resetChanges() {
    // Restore apps from the original copy
    apps = JSON.parse(JSON.stringify(originalApps));
    unsavedChanges = false;
    pool.unsavedApps = false;
  }

  async function loadApps() {
    loadingApps = true;
    try {
      apps = await getAppsForGym({ poolId: pool._id });
    } catch (error) {
      console.error('Failed to load apps:', error);
      apps = [];
    } finally {
      loadingApps = false;
    }
  }

  function updateUnsavedChanges() {
    unsavedChanges = true;
    pool.unsavedApps = true;
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
    } else if (editingField === 'price') {
      const price = parseFloat(editValue);
      if (!isNaN(price) && price >= 1) {
        pool.pricePerDemo = price;
        pool.unsavedPrice = true;
      }
    }

    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
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

    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
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
    apps = [...apps]; // Trigger reactivity

    // If we were moving a task to this new app
    if (currentTaskForAppChange) {
      changeTaskApp(apps.length - 1);
    }

    unsavedChanges = true;
    pool.unsavedApps = true;
    resetNewAppForm();
  }

  function resetNewAppForm() {
    newAppName = '';
    newAppDomain = '';
    showNewAppForm = false;
  }

  function addNewTask(appIndex: number) {
    const newTask = {
      prompt: "Enter task description here",
      type: "text"
    };
    
    apps[appIndex].tasks.push(newTask);
    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
    
    // Start editing the new task immediately
    const taskIndex = apps[appIndex].tasks.length - 1;
    startEditing(apps[appIndex].name, taskIndex.toString(), 'prompt', newTask.prompt);
  }
  
  function removeTask(appIndex: number, taskIndex: number) {
    apps[appIndex].tasks.splice(taskIndex, 1);
    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
  }
  
  function removeApp(appIndex: number) {
    apps.splice(appIndex, 1);
    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
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
  <div class="flex items-center gap-2">
    <div class="bg-gray-100 rounded-lg p-1 flex items-center">
      <button
        class="px-3 py-1.5 text-sm rounded-md transition-colors cursor-pointer flex items-center gap-1
          {viewMode === 'edit'
          ? 'bg-white text-secondary-600 shadow-sm'
          : 'text-gray-600 hover:text-gray-800'}"
        onclick={() => (viewMode = 'edit')}>
        <Pencil size={15} class="mr-1" />
        Edit
      </button>
      <button
        class="px-3 py-1.5 text-sm rounded-md cursor-pointer transition-colors flex items-center gap-1
          {viewMode === 'preview'
          ? 'bg-white text-secondary-600 shadow-sm'
          : 'text-gray-600 hover:text-gray-800'}"
        onclick={() => (viewMode = 'preview')}>
        <Eye size={15} class="mr-1" />
        Preview
      </button>
    </div>
    <button
      class="px-3 py-1.5 cursor-pointer text-sm rounded-md bg-secondary-300 text-white hover:bg-secondary-100 transition-colors flex items-center"
      onclick={regenerateTasks}>
      <Sparkles size={15} class="mr-1" />
      Generate Tasks
    </button>
  </div>
</div>

<!-- Scrollable task container -->
<div
  class="overflow-y-auto px-3 relative"
  style={`max-height: calc(100vh - ${(pool.status === TrainingPoolStatus.noFunds || pool.status === TrainingPoolStatus.noGas ? 120 : 0) + 220}px);`}>
  <!-- Tasks Tab Content -->
  {#if loadingApps}
    <div class="flex items-center justify-center h-40">
      <div
        class="animate-spin h-8 w-8 border-4 border-secondary-500 rounded-full border-t-transparent">
      </div>
    </div>
  {:else if viewMode === 'preview'}
    <!-- Preview Mode using AvailableTasks component -->
    <AvailableTasks
      {apps}
      {loadingApps}
      viewMode="preview"
      isGymBuilder={true} />
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
                  <button
                    class="w-8 h-8 flex items-center justify-center rounded bg-gray-100 cursor-pointer relative group"
                    onclick={() => startEditing(app.name, '', 'domain', app.domain)}>
                    {#if app.domain}
                      <img src={getIconUrl(app.domain)} alt={app.name} class="w-6 h-6" />
                    {:else}
                      <div class="w-6 h-6 bg-gray-300 rounded"></div>
                    {/if}
                    <div
                      class="absolute inset-0 bg-black/0 group-hover:bg-black/50 rounded flex items-center justify-center">
                      <Pencil size={12} class="text-white opacity-0 group-hover:opacity-100" />
                    </div>
                  </button>
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
                  <button
                    class="text-lg font-medium text-gray-800 group relative cursor-pointer px-1"
                    onclick={() => startEditing(app.name, '', 'name', app.name)}>
                    <span>{app.name}</span>
                    <div
                      class="absolute inset-0 bg-black/0 group-hover:bg-black/50 rounded flex items-center justify-center">
                      <Pencil size={12} class="text-white opacity-0 group-hover:opacity-100" />
                    </div>
                  </button>
                {/if}
              </div>

              <!-- App Actions -->
              <div class="flex items-center gap-2">
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
                  <button
                    class="text-sm font-bold text-secondary-600 flex items-center gap-1 group relative cursor-pointer"
                    onclick={() =>
                      startEditing(app.name, '', 'price', (pool.pricePerDemo || 1).toString())}>
                    <DollarSign size={14} />
                    {pool.pricePerDemo}
                    {pool.token.symbol}
                    <div
                      class="absolute inset-0 bg-black/0 group-hover:bg-black/50 rounded flex items-center justify-center">
                      <Pencil size={12} class="text-white opacity-0 group-hover:opacity-100" />
                    </div>
                  </button>
                {/if}
                
                <!-- Remove App Button -->
                <button 
                  class="text-gray-400 hover:text-red-500 p-1 ml-2 transition-colors" 
                  title="Remove App"
                  onclick={() => removeApp(appIndex)}>
                  <Trash2 size={16} />
                </button>
              </div>
            </div>

            <!-- App tasks section -->
            <div class="mt-4 pt-3">
              <div class="flex justify-between items-center mb-2">
                <div class="text-sm font-medium text-gray-700">Tasks</div>
                <button 
                  class="text-secondary-600 hover:text-secondary-800 flex items-center gap-1 text-xs"
                  onclick={() => addNewTask(appIndex)}>
                  <Plus size={14} />
                  Add Task
                </button>
              </div>

              {#if app.tasks && app.tasks.length > 0}
                <div class="space-y-2 ml-1">
                  {#each app.tasks as task, taskIndex}
                    {#if editingAppId === app.name && editingTaskId === taskIndex.toString() && editingField === 'prompt'}
                      <div class="mb-2">
                        <TextArea class="w-full text-sm" variant="light" bind:value={editValue}>
                        </TextArea>
                        <div class="flex justify-end mt-2 gap-2">
                          <button
                            class="text-green-500 hover:text-green-700 p-1"
                            onclick={saveEditing}>
                            <Check size={16} />
                          </button>
                          <button
                            class="text-red-500 hover:text-red-700 p-1"
                            onclick={cancelEditing}>
                            <X size={16} />
                          </button>
                        </div>
                      </div>
                    {:else}
                      <div class="flex items-center w-full group">
                        <button
                          class="w-full text-left cursor-pointer flex bg-gray-50 p-2 rounded-md hover:bg-gray-200 hover:shadow-sm transition-all duration-320"
                          onclick={() =>
                            startEditing(app.name, taskIndex.toString(), 'prompt', task.prompt)}>
                          <p class="grow text-gray-800 text-sm">{task.prompt}</p>
                          <Pencil
                            size={14}
                            class="text-gray-700 grow-0 opacity-0 group-hover:opacity-100 mt-1 transition-all duration-200" />
                        </button>
                        <button 
                          class="text-gray-400  opacity-0 group-hover:opacity-100 hover:text-red-500 p-1 ml-2 transition-all" 
                          title="Remove Task"
                          onclick={() => removeTask(appIndex, taskIndex)}>
                          <Trash2 size={14} />
                        </button>
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
    
    <!-- Add New App Button -->
    {#if !showNewAppForm}
      <div class="mt-4 flex justify-center">
        <button
          class="px-4 py-2 bg-secondary-500 text-white rounded-md hover:bg-secondary-600 transition-colors flex items-center gap-2"
          onclick={() => showNewAppForm = true}>
          <Plus size={16} />
          Add New App
        </button>
      </div>
    {:else}
      <Card padding="md" className="border border-gray-200 mt-4">
        <div class="p-2">
          <h3 class="text-lg font-medium mb-3">Add New App</h3>
          <div class="space-y-3">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">App Name</label>
              <input
                type="text"
                class="w-full px-3 py-2 border border-gray-300 rounded-md"
                placeholder="Enter app name"
                bind:value={newAppName} />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">App Domain (optional)</label>
              <input
                type="text"
                class="w-full px-3 py-2 border border-gray-300 rounded-md"
                placeholder="e.g. example.com"
                bind:value={newAppDomain} />
            </div>
            <div class="flex justify-end gap-2 pt-2">
              <button
                class="px-3 py-1.5 border border-gray-300 rounded-md hover:bg-gray-100"
                onclick={resetNewAppForm}>
                Cancel
              </button>
              <button
                class="px-3 py-1.5 bg-secondary-500 text-white rounded-md hover:bg-secondary-600 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={!newAppName}
                onclick={addNewApp}>
                Add App
              </button>
            </div>
          </div>
        </div>
      </Card>
    {/if}
  {/if}

    
  <!-- Sticky bottom buttons for save/reset -->
  {#if unsavedChanges}
  <div class="sticky bottom-0 w-full bg-gray-50 border-gray-200 p-4 z-10">
    <div class="flex gap-2">
      <Button
        class="flex-1 justify-center bg-green-500! text-white! hover:bg-green-600!"
        onclick={saveChanges}>
        <div class="flex items-center">
          <Save size={16} class="mr-2" />
          Save Changes
        </div>
      </Button>
      <Button
        class="flex-1 justify-center"
        variant="secondary"
        onclick={resetChanges}>
        <div class="flex items-center">
          <RotateCcw size={16} class="mr-2" />
          Reset Changes
        </div>
      </Button>
    </div>
  </div>

  {/if}

</div>
