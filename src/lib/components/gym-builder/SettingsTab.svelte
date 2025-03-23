<script lang="ts">
  import Input from '../Input.svelte';
  import Button from '../Button.svelte';
  import CopyButton from '../CopyButton.svelte';
  import Card from '../Card.svelte';
  import Checkbox from '../Checkbox.svelte';
  import { RefreshCw, Play, Pause, AlertTriangle } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import { onMount } from 'svelte';
  import { getPoolSubmissions, getAppsForGym } from '$lib/api/forge';
  import { writable } from 'svelte/store';
  import type { ForgeApp } from '$lib/types/gym';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
    unsavedUploadLimit?: boolean;
  };
  export let onSave: (pool: TrainingPool, updates: any) => Promise<void>;
  export let onRefresh: (poolId: string) => Promise<void>;
  export let refreshingPools: Set<string>;
  export let unsavedChanges: boolean;
  export let handleSaveChanges: () => Promise<void>;

  // Upload limit stats
  let currentCount = 0;
  let isLoadingStats = false;
  let loadError = false;
  let percentage = 0;
  
  // Task-specific data
  type TaskUploadCount = {
    taskId: string;
    appName: string;
    taskName: string;
    count: number;
  };
  
  let taskCounts: TaskUploadCount[] = [];
  let poolApps: ForgeApp[] = [];

  function handleNameChange(event: Event) {
    pool.name = (event.target as HTMLInputElement).value;
    pool.unsavedName = true;
    unsavedChanges = true;
  }

  function handlePriceChange(event: Event) {
    const price = parseFloat((event.target as HTMLInputElement).value);
    // Ensure price is at least 1 VIRAL
    pool.pricePerDemo = Math.max(1, price);
    pool.unsavedPrice = true;
    unsavedChanges = true;
  }

  // Upload limit handling
  let enableUploadLimit = !!pool.uploadLimit;
  let limitType: 'per-task' | 'per-day' | 'total' = (pool.uploadLimit?.limitType || 'per-task') as 'per-task' | 'per-day' | 'total';
  let limitValue = pool.uploadLimit?.type || 10;

  function handleEnableUploadLimitChange(event: Event) {
    enableUploadLimit = (event.target as HTMLInputElement).checked;
    if (enableUploadLimit) {
      pool.uploadLimit = {
        type: limitValue,
        limitType: limitType
      };
    } else {
      // Explicitly set to null to remove limits on the server
      pool.uploadLimit = null;
    }
    pool.unsavedUploadLimit = true;
    unsavedChanges = true;
  }

  function handleLimitTypeChange(event: Event) {
    limitType = (event.target as HTMLSelectElement).value as 'per-task' | 'per-day' | 'total';
    if (pool.uploadLimit && pool.uploadLimit !== null) {
      pool.uploadLimit.limitType = limitType;
      pool.unsavedUploadLimit = true;
      unsavedChanges = true;
      fetchUploadStats();
    }
  }

  function handleLimitValueChange(event: Event) {
    limitValue = parseInt((event.target as HTMLInputElement).value);
    if (pool.uploadLimit && pool.uploadLimit !== null) {
      pool.uploadLimit.type = limitValue;
      pool.unsavedUploadLimit = true;
      unsavedChanges = true;
      updatePercentage();
    }
  }

  function formatNumber(num: number) {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  async function fetchUploadStats() {
    if (!pool._id || !pool.uploadLimit || pool.uploadLimit === null) return;
    
    isLoadingStats = true;
    loadError = false;
    taskCounts = [];
    
    try {
      // Get submissions for this pool
      const submissions = await getPoolSubmissions(pool._id);
      
      if (limitType === 'total') {
        // Total uploads across the entire gym
        currentCount = submissions.length;
      } else if (limitType === 'per-day') {
        // Uploads per day across the entire gym
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        currentCount = submissions.filter(sub => {
          const subDate = new Date(sub.createdAt);
          return subDate >= today;
        }).length;
      } else if (limitType === 'per-task') {
        // For per-task, get all apps and tasks for this pool
        try {
          poolApps = await getAppsForGym({ poolId: pool._id });
          
          // Create a map of task IDs to task details
          const taskMap = new Map<string, { appName: string; taskName: string }>();
          poolApps.forEach(app => {
            app.tasks.forEach(task => {
              // Use a combination of app name and task prompt as the ID
              const taskId = `${app.name}:${task.prompt}`;
              taskMap.set(taskId, {
                appName: app.name,
                taskName: task.prompt
              });
            });
          });
          
          // Count submissions per task
          const countsMap = new Map<string, number>();
          
          submissions.forEach(sub => {
            const appName = sub.meta?.quest?.app;
            const taskName = sub.meta?.quest?.title;
            
            if (appName && taskName) {
              const taskId = `${appName}:${taskName}`;
              countsMap.set(taskId, (countsMap.get(taskId) || 0) + 1);
            }
          });
          
          // Convert to array for display
          taskCounts = Array.from(countsMap.entries()).map(([taskId, count]) => {
            const [appName, taskName] = taskId.split(':');
            return {
              taskId,
              appName,
              taskName,
              count
            };
          });
          
          // Sort by count (highest first)
          taskCounts.sort((a, b) => b.count - a.count);
          
          // For the progress bar, use the highest count
          currentCount = taskCounts.length > 0 ? taskCounts[0].count : 0;
        } catch (error) {
          console.error('Error fetching apps:', error);
          // Fallback to simple counting if we can't get app details
          const countsMap = new Map<string, number>();
          
          submissions.forEach(sub => {
            const taskId = sub.meta?.quest?.app_id;
            if (taskId) {
              countsMap.set(taskId, (countsMap.get(taskId) || 0) + 1);
            }
          });
          
          currentCount = countsMap.size > 0 ? Math.max(...countsMap.values()) : 0;
        }
      }
      
      updatePercentage();
    } catch (error) {
      console.error('Error fetching upload stats:', error);
      loadError = true;
    } finally {
      isLoadingStats = false;
    }
  }
  
  function updatePercentage() {
    if (limitValue > 0) {
      percentage = Math.min(100, Math.round((currentCount / limitValue) * 100));
    } else {
      percentage = 0;
    }
  }
  
  // Get progress bar color based on percentage
  $: progressColor = percentage >= 90 ? '#f44336' : // Red if ≥90%
                     percentage >= 75 ? '#ff9800' : // Orange if ≥75%
                     '#4caf50'; // Green otherwise
  
  onMount(() => {
    if (pool._id && pool.uploadLimit && pool.uploadLimit !== null) {
      fetchUploadStats();
    }
  });
</script>

<div class="space-y-6">
  <Card
    padding="md"
    className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1">Gym Name</label>
        <Input
          type="text"
          variant="light"
          value={pool.name}
          oninput={handleNameChange}
          placeholder="Enter gym name" />
      </div>

      <div>
        <div class="flex justify-between justify-items-center flex-row items-center mb-1">
          <label class="block text-sm font-medium">Deposit Address ({pool.token.symbol})</label>
          <Button
            class="px-2 py-1 text-xs"
            variant="secondary"
            onclick={() => onRefresh(pool._id)}
            disabled={refreshingPools.has(pool._id)}>
            <RefreshCw
              size={12}
              class={(refreshingPools.has(pool._id) ? 'animate-spin ' : '') + 'inline mb-1'} />
            Refresh Balance
          </Button>
        </div>
        <div class="flex gap-2 items-center">
          <input
            type="text"
            class="flex-1 p-2 bg-gray-100 rounded-lg cursor-text text-sm"
            readonly
            value={pool.depositAddress} />
          <CopyButton content={pool.depositAddress} />
        </div>
        <p class="text-xs text-gray-500 mt-1">
          Send {pool.token.symbol} tokens to this address to fund your gym.
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Price per Demonstration</label>
        <div class="flex items-center gap-2">
          <input
            type="number"
            min="1"
            step="0.1"
            class="flex-1 p-2 bg-gray-100 rounded-lg border-0"
            value={pool.pricePerDemo || 1}
            oninput={handlePriceChange} />
          <span class="text-sm font-medium">{pool.token.symbol}</span>
        </div>
        <p class="text-xs text-gray-500 mt-1">
          Minimum price: 1 {pool.token.symbol}
        </p>
      </div>

      <div class="p-3 bg-blue-50 rounded-lg border border-blue-100">
        <div class="flex justify-between mb-1">
          <span class="text-sm text-blue-800 font-medium">Current Balance:</span>
          <span class="text-sm font-bold text-blue-800">
            {formatNumber(pool.funds)}
            {pool.token.symbol}
          </span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-blue-800 font-medium">Possible Demos:</span>
          <span class="text-sm font-bold text-blue-800">
            {Math.floor(pool.funds / (pool.pricePerDemo || 1))}
          </span>
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Upload Limits</label>
        <div class="space-y-3">
          <Checkbox
            id="enableUploadLimit"
            variant="light"
            label="Enable gym-wide upload limits"
            checked={enableUploadLimit}
            onchange={handleEnableUploadLimitChange} />

          {#if enableUploadLimit}
            <div class="pl-6 space-y-3">
              <div class="space-y-3">
                <div class="flex items-center gap-4">
                  <div class="flex-1">
                    <label class="block text-sm font-medium mb-1">Limit Type</label>
                    <select
                      class="w-full p-2 bg-gray-100 rounded-lg border-gray-300 focus:ring-secondary-300 focus:border-secondary-300 focus:outline-none"
                      value={limitType}
                      onchange={handleLimitTypeChange}>
                      <option value="per-task">Per Task</option>
                      <option value="per-day">Per Day</option>
                      <option value="total">Total</option>
                    </select>
                  </div>
                  <div class="flex-1">
                    <label class="block text-sm font-medium mb-1">Limit Value</label>
                    <Input
                      type="number"
                      min="1"
                      step="1"
                      variant="light"
                      value={limitValue}
                      oninput={handleLimitValueChange} />
                  </div>
                </div>
                
                <p class="text-xs text-gray-500">
                  {#if limitType === 'per-task'}
                    Limit the number of uploads per individual task
                  {:else if limitType === 'per-day'}
                    Limit the number of uploads per day across all tasks
                  {:else}
                    Limit the total number of uploads across all tasks
                  {/if}
                </p>
                
                <!-- Upload Limit Progress Indicator -->
                {#if pool.uploadLimit}
                  <div class="mt-2 p-3 bg-gray-50 rounded-lg border border-gray-200">
                    {#if limitType !== 'per-task'}
                      <!-- Progress bar for per-day and total limits -->
                      <div class="flex justify-between mb-1">
                        <span class="text-sm font-medium">
                          {#if limitType === 'per-day'}
                            Daily Uploads
                          {:else}
                            Total Uploads
                          {/if}
                        </span>
                        <span class="text-sm font-bold">
                          {#if isLoadingStats}
                            <span class="text-gray-500">Loading...</span>
                          {:else}
                            <span class="current-count">{currentCount}</span>/{limitValue}
                          {/if}
                        </span>
                      </div>
                      <div class="h-2 bg-gray-200 rounded-full overflow-hidden">
                        <div 
                          class="h-full transition-all duration-300 ease-in-out" 
                          style="width: {percentage}%; background-color: {progressColor};"
                          data-limit-progress>
                        </div>
                      </div>
                      
                      {#if percentage >= 75}
                        <div class="flex items-center mt-2 text-xs" style="color: {progressColor};">
                          <AlertTriangle size={12} class="mr-1" />
                          {#if percentage >= 90}
                            <span>Critical: Upload limit almost reached!</span>
                          {:else}
                            <span>Warning: Approaching upload limit</span>
                          {/if}
                        </div>
                      {/if}
                    {/if}
                    
                    {#if loadError}
                      <p class="text-xs mt-1 text-red-500">
                        Error loading stats. <button class="underline" onclick={fetchUploadStats}>Retry</button>
                      </p>
                    {:else}
                      <p class="text-xs mt-1 text-gray-500">
                        {#if isLoadingStats}
                          Loading upload statistics...
                        {:else}
                          <button class="text-blue-500 underline" onclick={fetchUploadStats}>Refresh stats</button>
                        {/if}
                      </p>
                    {/if}
                    
                    <!-- Task-specific upload counts -->
                    {#if limitType === 'per-task' && !isLoadingStats}
                      <div class={loadError ? "mt-3" : ""}>
                        <h4 class="text-sm font-medium mb-2">Task Upload Breakdown</h4>
                        <div class="max-h-60 overflow-y-auto pr-1">
                          {#if taskCounts.length === 0}
                            <p class="text-sm text-gray-500 italic">No task uploads recorded yet.</p>
                          {:else}
                            <table class="w-full text-xs">
                              <thead class="text-left text-gray-500">
                                <tr>
                                  <th class="pb-1">App / Task</th>
                                  <th class="pb-1 w-[140px]">Uploads</th>
                                </tr>
                              </thead>
                              <tbody>
                                {#each taskCounts as task}
                                  <tr class="border-b border-gray-100 last:border-0">
                                    <td class="py-1.5">
                                      <div class="font-medium">{task.appName}</div>
                                      <div class="text-gray-500 truncate max-w-[200px]" title={task.taskName}>
                                        {task.taskName}
                                      </div>
                                    </td>
                                  <td class="py-1.5">
                                    <div class="flex flex-col gap-1">
                                      <div class="flex justify-between text-xs">
                                        <span class="font-medium">{task.count}/{limitValue}</span>
                                        <span class="text-xs" style="color: {task.count >= limitValue * 0.9 ? '#f44336' : 
                                                                          task.count >= limitValue * 0.75 ? '#ff9800' : 
                                                                          '#4caf50'}">
                                          {Math.round((task.count / limitValue) * 100)}%
                                        </span>
                                      </div>
                                      <div class="h-1.5 bg-gray-200 rounded-full overflow-hidden">
                                        <div 
                                          class="h-full transition-all duration-300 ease-in-out" 
                                          style="width: {Math.min(100, Math.round((task.count / limitValue) * 100))}%; 
                                                background-color: {task.count >= limitValue * 0.9 ? '#f44336' : 
                                                                  task.count >= limitValue * 0.75 ? '#ff9800' : 
                                                                  '#4caf50'};">
                                        </div>
                                      </div>
                                    </div>
                                  </td>
                                  </tr>
                                {/each}
                              </tbody>
                            </table>
                          {/if}
                        </div>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </Card>

  <div class="flex flex-col gap-2">
    {#if unsavedChanges}
      <Button
        class="w-full justify-center border-green-500! hover:border-green-600! bg-green-500! text-white! hover:bg-green-600!"
        onclick={handleSaveChanges}>
        Save Changes
      </Button>
    {/if}

    <Button
      behavior="none"
      class="py-4!"
      variant={pool.status === TrainingPoolStatus.live ? 'secondary' : 'green'}
      title={pool.status === TrainingPoolStatus.live ? 'Pause Gym' : 'Activate Gym'}
      onclick={() =>
        onSave(pool, {
          status:
            pool.status === TrainingPoolStatus.live
              ? TrainingPoolStatus.paused
              : TrainingPoolStatus.live
        })}>
      <div class="flex items-center">
        {#if pool.status === TrainingPoolStatus.live}
          <Pause size={16} class="mr-2" />
          Pause Gym
        {:else}
          <Play size={16} class="mr-2" />
          Activate Gym
        {/if}
      </div>
    </Button>
  </div>
</div>
