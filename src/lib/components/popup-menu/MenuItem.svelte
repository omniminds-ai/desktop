<script lang="ts">
  export let icon: any = null;
  export let danger = false;
</script>

<button
  on:click
  class="w-full cursor-pointer text-left px-3 py-2 hover:bg-gray-100 flex items-center gap-2 {danger
    ? 'text-red-500 hover:bg-red-50'
    : 'text-gray-700'}">
  {#if icon}
    <svelte:component this={icon} class="w-4 h-4" />
  {/if}
  <span><slot /></span>
</button>
