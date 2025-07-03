<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Screen } from '$lib/types/gym';
  import { X } from 'lucide-svelte';
  export let screens: Screen[] = [];
  export let onSelect: (id: string) => void;
  export let onClose: () => void;``

  function handleSelect(id: string) {
    onSelect(id);
    onClose();
  }

  function cleanupScreenName(name: string) {
    return name.replace(/[\\|.]*/, '').trim();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div
    class="bg-background-100 rounded-lg p-6 shadow-lg min-w-[350px] w-full max-h-[70vh] max-w-[90vw] relative">
    <button
      class="absolute top-3 right-3 text-gray-400 hover:text-white transition-colors"
      on:click={onClose}
      aria-label="Close modal">
      <X size={24} />
    </button>
    <h2 class="text-lg text-gray-200 mb-4">Select a screen to record</h2>
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
      {#each screens as screen}
        <button
          type="button"
          class="border-2 border-gray-300 rounded-lg overflow-hidden cursor-pointer transition-all hover:border-secondary-400 focus:border-secondary-500 focus:outline-none w-full text-left"
          on:click={() => handleSelect(screen.id)}
          on:keydown={(e) => e.key === 'Enter' && handleSelect(screen.id)}>
          <img src={screen.capture} alt="Screen preview" class="w-full h-full object-cover" />
          <span class="p-2 text-center text-m text-gray-200">
            {cleanupScreenName(screen.name)}
          </span>
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  /* Add any additional styles if needed */
</style> 