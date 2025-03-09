<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  
  export let open = false;
  export let position: { x: number; y: number } = { x: 0, y: 0 };
  
  const dispatch = createEventDispatcher();
  
  function handleClickOutside(event: MouseEvent) {
    if (open && !event.defaultPrevented) {
      dispatch('close');
    }
  }
  
  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    return () => {
      document.removeEventListener('click', handleClickOutside);
    };
  });
</script>

{#if open}
  <div 
    transition:fade={{ duration: 100 }}
    class="absolute z-50 bg-white rounded-lg shadow-lg border border-gray-200 py-1 min-w-[150px]"
    style="top: {position.y}px; left: {position.x}px;"
  >
    <slot />
  </div>
{/if}
