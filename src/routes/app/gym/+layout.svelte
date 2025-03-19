<script lang="ts">
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { connectionToken, walletAddress } from '$lib/stores/wallet';
  import { listSubmissions } from '$lib/api/forge';
  import { toolsInitState } from '$lib/utils';
  import ToastContainer from '$lib/components/ToastContainer.svelte';

  const { children } = $props();

  let pendingRecordings = [];
  let hasPendingRewards = $state(false);

  async function checkPendingRewards() {
    try {
      const recordings = await invoke<any[]>('list_recordings');
      let submissions: any[] = [];

      if ($walletAddress && $connectionToken) {
        submissions = await listSubmissions();
      }

      // Reset pending recordings
      pendingRecordings = [];

      // Check for completed recordings with rewards that haven't been submitted
      pendingRecordings = recordings.filter(
        (recording) =>
          recording.status === 'completed' &&
          recording.quest?.reward?.max_reward &&
          !submissions.some((s) => s.meta?.id === recording.id)
      );

      hasPendingRewards = pendingRecordings.length > 0;
    } catch (error) {
      console.error('Failed to load recordings:', error);
      // trigger signout since htis is probably from a 401 error
      $walletAddress = null;
      $connectionToken = null;
    }
  }

  onMount(() => {
    checkPendingRewards();

    // Set up interval to periodically check for pending rewards
    const interval = setInterval(checkPendingRewards, 5000);

    return () => {
      clearInterval(interval);
    };
  });

  // Subscribe to wallet address changes
  walletAddress.subscribe((val) => {
    if (val !== $walletAddress) {
      checkPendingRewards();
    }
  });
</script>

<div class="relative">
  <div class="p-2 sm:p-4">
    {#if $toolsInitState.initializing}
      <div class="px-4 rounded my-2 max-w-7xl mx-auto py-2 flex flex-col">
        <div class="flex justify-between items-center mb-1">
          <span class="text-sm font-medium">Installing necessary demonstration tools...</span>
          <span class="text-xs text-gray-500">{$toolsInitState.progress}%</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-1.5">
          <div
            class="bg-secondary-400 h-1.5 rounded-full transition-all duration-300 ease-in-out"
            style="width: {$toolsInitState.progress}%">
          </div>
        </div>
      </div>
    {/if}
    <GymHeader {hasPendingRewards} />
    {@render children()}
  </div>
</div>

<ToastContainer />
