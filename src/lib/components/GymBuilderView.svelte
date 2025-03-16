<script lang="ts">
  import Button from './Button.svelte';
  import GenerateGymModal from './GenerateGymModal.svelte';
  import { Upload, ListTodo, Sliders, LayoutDashboard } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';

  // Import our tab components
  import OverviewTab from './gym-builder/OverviewTab.svelte';
  import SettingsTab from './gym-builder/SettingsTab.svelte';
  import TasksTab from './gym-builder/TasksTab.svelte';
  import UploadsTab from './gym-builder/UploadsTab.svelte';

  // Props
  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
    unsavedApps?: boolean;
  };
  export let onSave: (pool: TrainingPool, updates: any) => Promise<void>;
  export let onRefresh: (poolId: string) => Promise<void>;
  export let refreshingPools: Set<string>;
  export let viralPrice: number = 0;

  let activeTab = 'overview';
  let unsavedChanges = false;
  let showSkillsModal = false;

  // Reference to the TasksTab component to access its apps array
  let tasksTabComponent: TasksTab | null = null;
  // Direct binding for the apps array from TasksTab
  let tasksApps: any[] = [];
  
  async function handleSaveChanges() {
    try {
      // Create an updates object to hold all changes
      const updates: any = {};
      
      console.log('Saving changes, activeTab:', activeTab, 'unsavedChanges:', unsavedChanges);
      
      // Add skills if they've changed
      if (pool.unsavedSkills) {
        updates.skills = pool.skills;
        pool.unsavedSkills = false;
      }
      
      // Add price if it's changed
      if (pool.unsavedPrice) {
        updates.pricePerDemo = pool.pricePerDemo;
        pool.unsavedPrice = false;
      }
      
      // Add name if it's changed
      if (pool.unsavedName) {
        updates.name = pool.name;
        pool.unsavedName = false;
      }
      
      // Add apps if they've changed
      if (pool.unsavedApps) {
        if (!tasksApps || !Array.isArray(tasksApps)) {
          console.error('Invalid apps array:', tasksApps);
          throw new Error('Apps array is invalid or undefined');
        }
        console.log('Adding apps to updates:', tasksApps);
        updates.apps = tasksApps;
        pool.unsavedApps = false;
      }
      
      // Only save if there are updates to make
      if (Object.keys(updates).length > 0) {
        console.log('Saving updates:', updates);
        await onSave(pool, updates);
      } else {
        console.log('No updates to save');
      }
      
      unsavedChanges = false;
      showSkillsModal = false;
    } catch (error) {
      console.error('Failed to save changes:', error);
      alert('Failed to save changes. Please try again.');
    }
  }

  function setTab(tab: string) {
    activeTab = tab;
  }

  async function regenerateTasks() {
    showSkillsModal = true;
  }

  function handleGymModalChange(event: CustomEvent) {
    pool.skills = event.detail.skills;
    pool.unsavedSkills = true;
    unsavedChanges = true;
  }

  async function handleGymModalSave(event: CustomEvent) {
    try {
      const { generatedResponse } = event.detail || {};

      if (
        generatedResponse &&
        generatedResponse.content &&
        generatedResponse.content.apps &&
        generatedResponse.content.apps.length > 0
      ) {
        // Save the generated apps along with the skills
        const apps = generatedResponse.content.apps;
        
        // First save the skills
        if (pool.unsavedSkills) {
          await onSave(pool, { 
            skills: pool.skills,
            apps: apps  // Pass the apps array along with skills
          });
          pool.unsavedSkills = false;
          pool.unsavedApps = false;
        }
        
        // Then save other changes if needed
        if (pool.unsavedPrice) {
          await onSave(pool, { pricePerDemo: pool.pricePerDemo });
          pool.unsavedPrice = false;
        }

        if (pool.unsavedName) {
          await onSave(pool, { name: pool.name });
          pool.unsavedName = false;
        }
        
        unsavedChanges = false;
        showSkillsModal = false;
      } else {
        // If no apps were generated, just save other changes
        await handleSaveChanges();
      }
    } catch (error) {
      console.error('Failed to save generated apps:', error);
      // Fallback to regular save
      handleSaveChanges();
    }
  }
</script>

<!-- No fixed overlay, just the content -->
<div class="flex flex-col h-full pt-2">
  <!-- Tab Navigation with Pause/Activate Button as Icon -->
  <div class="relative flex flex-col flex-wrap gap-1 border-b border-gray-200 mb-4">
    <div class="flex flex-row">
      <div class="flex items-center w-full gap-3">
        <h1
          class="text-2xl font-bold text-gray-800 mr-3 truncate overflow-hidden text-ellipsis max-w-[calc(100vw-8em)]"
          title={pool.name}>
          {pool.name}
        </h1>
        <!-- Power Button (Pause/Activate) with custom SVG -->
        <Button
          behavior="none"
          variant={pool.status === TrainingPoolStatus.live ? 'secondary' : 'green'}
          title={pool.status === TrainingPoolStatus.live ? 'Pause Gym' : 'Activate Gym'}
          onclick={() =>
            onSave(pool, {
              status:
                pool.status === TrainingPoolStatus.live
                  ? TrainingPoolStatus.paused
                  : TrainingPoolStatus.live
            })}>
          {#if pool.status === TrainingPoolStatus.live}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg">
              <rect x="6" y="4" width="4" height="16" fill="currentColor" />
              <rect x="14" y="4" width="4" height="16" fill="currentColor" />
            </svg>
          {:else}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg">
              <path d="M5 3L19 12L5 21V3Z" fill="currentColor" />
            </svg>
          {/if}
        </Button>
        <!-- Status Indicator -->
        <div
          class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full {pool.status ===
          TrainingPoolStatus.live
            ? 'bg-green-500/10 text-green-500'
            : pool.status === TrainingPoolStatus.paused
              ? 'bg-gray-500/10 text-gray-500'
              : 'bg-yellow-500/10 text-yellow-600'}">
          <div
            class="w-1.5 h-1.5 rounded-full {pool.status === TrainingPoolStatus.live
              ? 'bg-green-500 animate-pulse'
              : pool.status === TrainingPoolStatus.paused
                ? 'bg-gray-500'
                : 'bg-yellow-500 animate-pulse'}">
          </div>
          {pool.status}
        </div>
      </div>
    </div>
    <div class="flex flex-row gap-1">
      <button
        class="px-4 py-2 text-sm font-medium cursor-pointer transition-colors flex items-center gap-1
          {activeTab === 'overview'
          ? 'text-secondary-600 border-b-2 border-secondary-500'
          : 'text-gray-500 hover:text-gray-700'}"
        onclick={() => setTab('overview')}>
        <LayoutDashboard size={16} />
        Overview
      </button>
      <button
        class="px-4 py-2 text-sm font-medium cursor-pointer transition-colors flex items-center gap-1
          {activeTab === 'settings'
          ? 'text-secondary-600 border-b-2 border-secondary-500'
          : 'text-gray-500 hover:text-gray-700'}"
        onclick={() => setTab('settings')}>
        <Sliders size={16} />
        Settings
      </button>
      <button
        class="px-4 py-2 text-sm cursor-pointer font-medium transition-colors flex items-center gap-1
          {activeTab === 'tasks'
          ? 'text-secondary-600 border-b-2 border-secondary-500'
          : 'text-gray-500 hover:text-gray-700'}"
        onclick={() => setTab('tasks')}>
        <ListTodo size={16} />
        Tasks
      </button>
      <button
        class="px-4 py-2 text-sm font-medium cursor-pointer transition-colors flex items-center gap-1
          {activeTab === 'uploads'
          ? 'text-secondary-600 border-b-2 border-secondary-500'
          : 'text-gray-500 hover:text-gray-700'}"
        onclick={() => setTab('uploads')}>
        <Upload size={16} />
        Uploads
      </button>
    </div>
  </div>

  <!-- Tab Content -->
  <div class="flex-grow overflow-y-auto">
    {#if activeTab === 'overview'}
      <OverviewTab {pool} />
    {:else if activeTab === 'settings'}
      <SettingsTab
        {pool}
        {onSave}
        {onRefresh}
        {refreshingPools}
        {unsavedChanges}
        {handleSaveChanges} />
    {:else if activeTab === 'tasks'}
      <TasksTab 
        bind:this={tasksTabComponent}
        bind:apps={tasksApps}
        {pool} 
        bind:unsavedChanges 
        {regenerateTasks} 
        {handleSaveChanges} />
    {:else if activeTab === 'uploads'}
      <UploadsTab {pool} />
    {/if}
  </div>
</div>

<!-- Use the new GenerateGymModal component -->
<GenerateGymModal
  show={showSkillsModal}
  skills={pool.skills || ''}
  on:change={handleGymModalChange}
  on:close={() => (showSkillsModal = false)}
  on:save={handleGymModalSave} />
