<script lang="ts">
  import Button from '../Button.svelte';
  import TextArea from '../TextArea.svelte';
  import Input from '../Input.svelte';
  import Checkbox from '../Checkbox.svelte';
  import type { ForgeApp } from '$lib/types/gym';
  import Card from '../Card.svelte';

  export let show = false;
  export let app: ForgeApp | null = null;
  export let taskIndex = -1;
  export let onClose: () => void;
  export let onSave: () => void;
  
  export let prompt = '';
  export let enableUploadLimit = false;
  export let uploadLimitValue = 10;
  export let enableRewardLimit = false;
  export let rewardLimitValue = 10;

  function handleUploadLimitChange(event: Event) {
    enableUploadLimit = (event.target as HTMLInputElement).checked;
  }

  function handleRewardLimitChange(event: Event) {
    enableRewardLimit = (event.target as HTMLInputElement).checked;
  }

  // Close modal when clicking outside
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  $: isNewTask = taskIndex < 0;
</script>

{#if show && app}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={handleBackdropClick}
  >
    <div class="bg-white rounded-lg shadow-lg p-6 max-w-2xl w-full mx-4">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-semibold">{isNewTask ? 'Add New Task' : 'Edit Task'}</h2>
        <button 
          class="text-gray-500 hover:text-gray-700" 
          onclick={onClose}
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Task Prompt</label>
          <TextArea
            class="w-full"
            variant="light"
            bind:value={prompt}
            placeholder="Enter task description" />
        </div>

      <div>
        <Checkbox
          id="enableUploadLimit"
          variant="light"
          label="Enable upload limit for this task"
          checked={enableUploadLimit}
          onchange={handleUploadLimitChange} />

        {#if enableUploadLimit}
          <div class="pl-6 mt-2">
            <label class="block text-sm font-medium text-gray-700 mb-1">Upload Limit</label>
            <Input
              type="number"
              min="1"
              step="1"
              variant="light"
              bind:value={uploadLimitValue} />
            <p class="text-xs text-gray-500 mt-1">
              Maximum number of uploads allowed for this task
            </p>
          </div>
        {/if}
      </div>

      <div>
        <Checkbox
          id="enableRewardLimit"
          variant="light"
          label="Enable custom reward limit for this task"
          checked={enableRewardLimit}
          onchange={handleRewardLimitChange} />

        {#if enableRewardLimit}
          <div class="pl-6 mt-2">
            <label class="block text-sm font-medium text-gray-700 mb-1">Reward Limit</label>
            <div class="flex items-center gap-2">
              <Input
                type="number"
                min="1"
                step="1"
                variant="light"
                bind:value={rewardLimitValue} />
              <span class="text-sm font-medium">VIRAL</span>
            </div>
            <p class="text-xs text-gray-500 mt-1">
              Maximum reward amount for this task (overrides gym-wide price per demo)
            </p>
          </div>
        {/if}
      </div>
      </div>

      <div class="flex justify-end gap-2 pt-4 mt-4 border-t border-gray-200">
        <Button variant="secondary" onclick={onClose}>
          Cancel
        </Button>
        <Button variant="primary" onclick={onSave}>
          {isNewTask ? 'Add Task' : 'Save Changes'}
        </Button>
      </div>
    </div>
  </div>
{/if}
