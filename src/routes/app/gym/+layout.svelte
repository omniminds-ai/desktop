<script lang="ts">
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { walletAddress } from '$lib/stores/wallet';
  import { listSubmissions } from '$lib/api/forge';

  const { children } = $props();
  
  let pendingRecordings = [];
  let hasPendingRewards = $state(false);
  
  async function checkPendingRewards() {
    try {
      const recordings = await invoke<any[]>('list_recordings');
      let submissions: any[] = [];
      
      if ($walletAddress) {
        submissions = await listSubmissions();
      }
      
      // Reset pending recordings
      pendingRecordings = [];
      
      // Check for completed recordings with rewards that haven't been submitted
      pendingRecordings = recordings.filter(recording => 
        recording.status === 'completed' && 
        recording.quest?.reward?.max_reward &&
        !submissions.some(s => s.meta?.id === recording.id)
      );
      
      hasPendingRewards = pendingRecordings.length > 0;
    } catch (error) {
      console.error('Failed to load recordings:', error);
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
    <GymHeader hasPendingRewards={hasPendingRewards} />
    {@render children()}
  </div>
</div>
