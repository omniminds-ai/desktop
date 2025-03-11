<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import '../../app.css';
  import { checkForUpdate, updateApp } from '$lib/utils';
  import Button from '$lib/components/Button.svelte';

  let { children } = $props();

  let canceled = $state(false);
</script>

<div class="h-screen flex bg-primary-600/75 overflow-hidden backdrop-blur-md">
  <Sidebar />
  <div class="flex-1 p-2">
    <div class="h-full overflow-hidden bg-gray-50 border border-gray-500 rounded-md">
      <div class="h-full w-full overflow-auto">
        {@render children()}
      </div>
    </div>
  </div>
</div>

{#await checkForUpdate() then update}
  {#if update && !canceled}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-lg shadow-lg p-6 max-w-md w-full">
        <h2 class="text-xl font-semibold mb-4">New Update!</h2>
        <p class="text-gray-700 mb-6">
          A new version of Viralmind Desktop is available. Please update for new features, bug
          fixes, and other goodness.
        </p>
        <div class="flex justify-end gap-3">
          <Button variant="secondary" onclick={() => (canceled = true)}>Ignore</Button>
          <Button variant="primary" onclick={() => updateApp(update)}>Update</Button>
        </div>
      </div>
    </div>
  {/if}
{/await}

<style>
  :global(html, body) {
    background: transparent;
  }
</style>
