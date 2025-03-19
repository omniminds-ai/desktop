<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import Button from './Button.svelte';
  import { toasts, type Toast } from '$lib/stores/toast';

  const { toast } = $props<{
    toast: Toast;
  }>();

  function handleClose() {
    toasts.remove(toast.id);
  }

  function handleButtonClick() {
    if (toast.button?.action) {
      toast.button.action();
    }
    handleClose();
  }
</script>

<div
  class="bg-white rounded-lg shadow-md w-full max-w-sm mb-4 overflow-hidden border border-gray-200"
  in:fly={{ x: 300, duration: 300 }}
  out:fade={{ duration: 200 }}>
  <div class="p-4">
    <div class="flex justify-between gap-4 items-center mb-2">
      <h3 class="font-semibold text-base m-0">{toast.title}</h3>
      <button
        class="bg-transparent border-none cursor-pointer p-1 flex items-center justify-center text-gray-500 rounded hover:bg-gray-100"
        onclick={handleClose}
        aria-label="Close toast">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
    <div class="text-gray-600 mb-3 break-all">{toast.message}</div>
    {#if toast.button}
      <div class="flex justify-end">
        <Button
          variant={toast.button.variant || 'primary'}
          behavior="none"
          onclick={handleButtonClick}>
          {toast.button.text}
        </Button>
      </div>
    {/if}
  </div>
</div>
