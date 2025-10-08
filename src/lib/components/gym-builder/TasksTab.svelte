<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import TaskEditModal from '../modals/TaskEditModal.svelte';
  import {
    Pencil,
    Check,
    X,
    Eye,
    Sparkles,
    Plus,
    Trash2,
    Save,
    RotateCcw,
    Video
  } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import type { ForgeApp } from '$lib/types/gym';
  import { getAppsForGym } from '$lib/api/endpoints/forge';
  import AvailableTasks from '$lib/components/gym/AvailableTasks.svelte';
  import { onMount } from 'svelte';
  import Input from '../form/Input.svelte';

  // Store original apps for reset functionality
  let originalApps: ForgeApp[] = [];

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
    unsavedApps?: boolean;
  };
  export let unsavedChanges: boolean;
  export let regenerateTasks: () => void;

  let viewMode = 'edit'; // edit or preview
  // App editing variables
  let editingAppId = '';
  let editingTaskId = '';
  let editingField = '';
  let editValue = '';

  // Task editing modal
  let showTaskModal = false;
  let currentApp: ForgeApp | null = null;
  let currentTaskIndex = -1;
  let editTaskPrompt = '';
  let enableUploadLimit = false;
  let uploadLimitValue = 10;
  let enableRewardLimit = false;
  let rewardLimitValue = 10;
  let currentTaskForAppChange: { appIndex: number; taskIndex: number } | null = null;
  let newAppName = '';
  let newAppDomain = '';
  let showNewAppForm = false;
  let scrollableContainer: HTMLDivElement;
  let newAppFormRef: HTMLDivElement;
  // Make apps public so it can be accessed by the parent component
  export let apps: ForgeApp[] = [];
  let loadingApps = true;

  onMount(async () => {
    await loadApps();
    // Store a deep copy of the original apps
    originalApps = JSON.parse(JSON.stringify(apps));
  });

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

  function openTaskModal(app: ForgeApp, taskIndex: number) {
    currentApp = app;
    currentTaskIndex = taskIndex;

    if (taskIndex >= 0 && taskIndex < app.tasks.length) {
      const task = app.tasks[taskIndex];
      editTaskPrompt = task.prompt;
      enableUploadLimit = typeof task.uploadLimit === 'number';
      uploadLimitValue = task.uploadLimit ?? 10;
      enableRewardLimit = typeof task.rewardLimit === 'number';
      rewardLimitValue = task.rewardLimit ?? 10;
    } else {
      // New task
      editTaskPrompt = '';
      enableUploadLimit = false;
      uploadLimitValue = 10;
      enableRewardLimit = false;
      rewardLimitValue = 10;
    }

    showTaskModal = true;
  }

  function closeTaskModal() {
    showTaskModal = false;
    currentApp = null;
    currentTaskIndex = -1;
  }

  function saveTaskModal() {
    if (!currentApp) return;

    const appIndex = apps.findIndex((app) => app.name === currentApp!.name);
    if (appIndex === -1) return;

    if (currentTaskIndex >= 0 && currentTaskIndex < apps[appIndex].tasks.length) {
      // Update existing task
      apps[appIndex].tasks[currentTaskIndex].prompt = editTaskPrompt;
      apps[appIndex].tasks[currentTaskIndex].uploadLimit = enableUploadLimit
        ? uploadLimitValue
        : undefined;
      apps[appIndex].tasks[currentTaskIndex].rewardLimit = enableRewardLimit
        ? rewardLimitValue
        : undefined;

      // Log the updated task for debugging
      console.log('Updated task with limits:', {
        prompt: editTaskPrompt,
        uploadLimit: apps[appIndex].tasks[currentTaskIndex].uploadLimit,
        rewardLimit: apps[appIndex].tasks[currentTaskIndex].rewardLimit
      });
    } else {
      // Add new task
      const newTask = {
        prompt: editTaskPrompt,
        uploadLimit: enableUploadLimit ? uploadLimitValue : undefined,
        rewardLimit: enableRewardLimit ? rewardLimitValue : undefined
      };
      apps[appIndex].tasks.push(newTask);

      // Log the new task for debugging
      console.log('Added new task with limits:', newTask);
    }

    apps = [...apps]; // Trigger reactivity
    unsavedChanges = true;
    pool.unsavedApps = true;
    closeTaskModal();
  }

  function handleUploadLimitChange(event: Event) {
    enableUploadLimit = (event.target as HTMLInputElement).checked;
  }

  function handleRewardLimitChange(event: Event) {
    enableRewardLimit = (event.target as HTMLInputElement).checked;
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

  // Watch for changes to showNewAppForm and scroll to the form when it becomes true
  $: if (showNewAppForm && newAppFormRef) {
    // Use setTimeout to ensure the DOM has updated
    setTimeout(() => {
      newAppFormRef.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }, 0);
  }

  function addNewTask(appIndex: number) {
    if (appIndex >= 0 && appIndex < apps.length) {
      openTaskModal(apps[appIndex], -1); // -1 indicates a new task
    }
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
    <h2 class="text-xl font-bold text-gray-200">Tasks</h2>
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
      Regenerate Tasks
    </button>
  </div>
</div>

<!-- Scrollable task container -->
<div
  bind:this={scrollableContainer}
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
    <AvailableTasks {loadingApps} viewMode="preview" isGymBuilder={true} poolId={pool._id} />
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
                          class="w-8 h-8 flex items-center justify-center rounded bg-transparent cursor-pointer relative group"
                    onclick={() => startEditing(app.name, '', 'domain', app.domain)}>
                    {#if app.domain}
                      <img src={getIconUrl(app.domain)} alt={app.name} class="w-6 h-6" />
                    {:else}
                      <div class="w-6 h-6 bg-transparent rounded"></div>
                    {/if}
                    <div
                            class="absolute inset-0 bg-black/0 group-hover:bg-background-100 rounded flex items-center justify-center">
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
                          class="text-lg font-medium text-gray-200 group relative cursor-pointer px-1"
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
                <div class="text-sm font-medium text-gray-200">Tasks</div>
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
                    <div class="flex items-center w-full group">
                      <div
                              class="w-full text-left flex items-center bg-background-100 p-2 rounded-md hover:bg-gray-800 hover:shadow-sm transition-all duration-320">
                        <p class="grow text-gray-200 text-sm">{task.prompt}</p>
                        <div class="flex items-center gap-2 ml-2">
                          {#if typeof task.uploadLimit === 'number'}
                            <span
                              class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                              {task.currentSubmissions ?? 0}/{task.uploadLimit}
                            </span>
                          {/if}
                          {#if typeof task.rewardLimit === 'number'}
                            <span
                              class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                              {task.rewardLimit} OMNIS
                            </span>
                          {/if}
                          <a
                            href="/app/gym/chat?prompt={encodeURIComponent(
                              task.prompt
                            )}&app={encodeURIComponent(
                              JSON.stringify({
                                type: 'website',
                                name: app.name,
                                url: `https://${app.domain}`,
                                task_id: task._id
                              })
                            )}&poolId={app.pool_id._id}"
                            class="text-gray-700 opacity-0 group-hover:opacity-100 transition-all duration-200"
                            title="Record Task">
                            <Video size={14} />
                          </a>
                          <button
                            class="text-gray-700 opacity-0 group-hover:opacity-100 transition-all duration-200"
                            title="Edit Task"
                            onclick={() => openTaskModal(app, taskIndex)}>
                            <Pencil size={14} />
                          </button>
                        </div>
                      </div>
                      <button
                        class="text-gray-400 opacity-0 group-hover:opacity-100 hover:text-red-500 p-1 ml-2 transition-all"
                        title="Remove Task"
                        onclick={() => removeTask(appIndex, taskIndex)}>
                        <Trash2 size={14} />
                      </button>
                    </div>
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
          onclick={() => {
            showNewAppForm = true;
          }}>
          <Plus size={16} />
          Add New App
        </button>
      </div>
    {:else}
      <div bind:this={newAppFormRef}>
        <Card padding="md" className="border border-gray-200 mt-4">
          <div class="p-2">
            <h3 class="text-lg font-medium mb-3">Add New App</h3>
            <div class="space-y-3">
              <div>
                <label for="app-name" class="block text-sm font-medium text-gray-700 mb-1">
                  App Name
                </label>
                <Input
                  variant="light"
                  id="app-name"
                  type="text"
                  class="w-full px-3 py-2 border border-gray-300 rounded-md"
                  placeholder="Enter app name"
                  bind:value={newAppName} />
              </div>
              <div>
                <label for="app-domain" class="block text-sm font-medium text-gray-700 mb-1">
                  App Domain (optional)
                </label>
                <Input
                  variant="light"
                  id="app-domain"
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
      </div>
    {/if}
  {/if}
</div>

<!-- Task Editing Modal -->
<TaskEditModal
  show={showTaskModal}
  app={currentApp}
  taskIndex={currentTaskIndex}
  bind:prompt={editTaskPrompt}
  bind:enableUploadLimit
  bind:uploadLimitValue
  bind:enableRewardLimit
  bind:rewardLimitValue
  onClose={closeTaskModal}
  onSave={saveTaskModal} />
