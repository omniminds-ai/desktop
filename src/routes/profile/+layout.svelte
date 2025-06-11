<script lang="ts">
  import { confirm } from '@tauri-apps/plugin-dialog';
  import InitToolsFailedModal from '$lib/components/modals/InitToolsFailedModal.svelte';
  import ToastContainer from '$lib/components/toast/ToastContainer.svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import '../../app.css';
  import { checkForUpdate, toolsInitState, updateApp } from '$lib/utils';
  import Button from '$lib/components/form/Button.svelte';
  import { platform } from '@tauri-apps/plugin-os';
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  let { children } = $props();

  let canceled = $state(false);
  let timer: number | undefined;
  let unlistenInitErrors: UnlistenFn | null = null;
  let initErrors: string[] = $state([]);
  let showInitToolsError = $state(false);

  async function checkToolsStatus() {
    try {
      const status = await invoke<{
        ffmpeg: boolean;
        ffprobe: boolean;
        dump_tree: boolean;
        pipeline: boolean;
      }>('check_tools');

      // Calculate percentage of initialized tools
      const totalTools = 4; // ffmpeg, ffprobe, dump_tree, pipeline
      const initializedTools = Object.values(status).filter(Boolean).length;
      const progress = Math.round((initializedTools / totalTools) * 100);

      $toolsInitState.progress = progress || 5;

      // If all tools are initialized, stop checking
      if (initializedTools === totalTools) {
        console.log('All tools initialized');

        // Hide the progress bar after a short delay
        setTimeout(() => {
          $toolsInitState.initializing = false;
        }, 500);

        // Clear the interval if it's still active
        if (timer) {
          clearInterval(timer);
          timer = undefined;
        }

        return true;
      } else {
        $toolsInitState.initializing = true;
      }

      return false;
    } catch (error) {
      console.error('Failed to check tools status:', error);
      return false;
    }
  }

  onMount(async () => {
    // Initialize ffmpeg, ffprobe, dump-tree, and pipeline
    unlistenInitErrors = await listen(
      'init_tools_errors',
      async (event: { event: string; id: number; payload: { errors: string[] } }) => {
        initErrors = event.payload.errors;
        showInitToolsError = true;
        await confirm('Initializing tools failed', { cancelLabel: 'Ignore', okLabel: 'Blow up!' });
      }
    );

    // Start the tools initialization process
    invoke('init_tools');
    // Check tools status immediately
    checkToolsStatus();

    // Set up interval to check tools status every 2 seconds
    timer = setInterval(checkToolsStatus, 2000);
  });

  onDestroy(() => {
    unlistenInitErrors?.();
    // Clean up timer if component is destroyed
    if (timer) {
      clearInterval(timer);
    }
  });

  // Detect platform (windows or darwin for Mac)
  const currentPlatform = platform();
  const bgClass = currentPlatform === 'macos' ? 'bg-gray-800/50' : 'bg-gray-800';
</script>

<div class="h-screen flex {bgClass} overflow-hidden">
  <Sidebar />
  <div class="flex-1 p-2">
    <div class="h-full overflow-hidden bg-gray-800 rounded-md">
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
          A new version of Omniminds Desktop is available. Please update to get the latest updates, bugs fixes and new features

        </p>
        <div class="flex justify-end gap-3">
          <Button variant="secondary" onclick={() => (canceled = true)}>Ignore</Button>
          <Button variant="primary" onclick={() => updateApp(update)}>Update</Button>
        </div>
      </div>
    </div>
  {/if}
{/await}

<InitToolsFailedModal
  errors={initErrors}
  retry={async () => {
    invoke('init_tools');
  }}
  bind:open={showInitToolsError} />

<ToastContainer />

<style>
  :global(html, body) {
    background: transparent;
  }
</style>
