<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { onMount } from 'svelte';
  import { getBalance, listSubmissions } from '$lib/api/endpoints/forge';
  import Button from '$lib/components/form/Button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { walletAddress } from '$lib/stores/wallet';
  import { page } from '$app/state';
  import AvailableTasks from '$lib/components/gym/AvailableTasks.svelte';
  import { Wallet, History, Zap } from 'lucide-svelte';
  import WalletButton from '$lib/components/WalletButton.svelte';
  import { type ApiRecording, type LocalRecording, type Quest } from '$lib/types/gym';

  const poolId = page.url.searchParams.get('poolId') || undefined;
  const poolName = page.url.searchParams.get('poolName');

  function formatNumber(num: number): string {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  let loadingApps = true;
  let viralBalance = 0;
  let unclaimedRewards = 0;
  let recentSubmissions: any[] = [];
  let earnedThisMonth = 0;
  let pendingRecordings: LocalRecording[] = [];

  // Calculate total rewards from submissions
  function calculateEarnedRewards(submissions: any[]): number {
    return submissions.reduce((total, submission) => {
      if (submission.reward) {
        return total + submission.reward;
      }
      return total;
    }, 0);
  }
  async function loadBalance(address: string) {
    try {
      viralBalance = await getBalance(address);
      recentSubmissions = await listSubmissions();
      earnedThisMonth = calculateEarnedRewards(recentSubmissions);
    } catch (error) {
      console.error('Failed to load balance:', error);
    }
  }

  async function loadUnclaimedRewards() {
    try {
      const recordings = await invoke<LocalRecording[]>('list_recordings');
      let submissions: any[] = [];
      if ($walletAddress) {
        submissions = await listSubmissions();
      }

      // Reset pending recordings
      pendingRecordings = [];

      unclaimedRewards = recordings.reduce((total, recording) => {
        // Only count if recording is completed, has a reward, and hasn't been submitted
        if (
          recording.status === 'completed' &&
          recording.quest?.reward?.max_reward &&
          !submissions.some((s) => s.meta?.id === recording.id)
        ) {
          // Add to pending recordings list
          pendingRecordings.push(recording);
          return total + recording.quest.reward.max_reward;
        }
        return total;
      }, 0);
    } catch (error) {
      console.error('Failed to load recordings:', error);
    }
  }

  onMount(async () => {
    loadUnclaimedRewards();
    if ($walletAddress) {
      loadBalance($walletAddress);
    }
  });

  // Subscribe to wallet address changes
  walletAddress.subscribe((val) => {
    if (!$walletAddress || !val) {
      // Reset balance and reload unclaimed rewards when wallet disconnected
      viralBalance = 0;
      loadUnclaimedRewards();
    } else if (val !== $walletAddress) {
      loadBalance(val);
    }
  });

  // Calculate average earnings per submission
  $: averageEarnings =
    recentSubmissions.length > 0 ? earnedThisMonth / recentSubmissions.length : 0;
</script>

<div class="h-full mx-auto overflow-x-hidden">
  {#if poolName}
    <div class="mx-auto mb-8">
      <div class="text-xl font-semibold mb-2">Viewing Forge: {poolName}</div>
    </div>
  {/if}

  <div class={`${!$walletAddress ? 'blur-sm pointer-events-none' : ''}`}>
    <AvailableTasks {poolId} {loadingApps} isGymBuilder={false} />
  </div>
</div>
