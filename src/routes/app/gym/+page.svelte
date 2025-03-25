<script lang="ts">
  import Card from '$lib/components/Card.svelte';
  import { onMount } from 'svelte';
  import { getBalance, listSubmissions } from '$lib/api/forge';
  import Button from '$lib/components/form/Button.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { walletAddress } from '$lib/stores/wallet';
  import { page } from '$app/state';
  import AvailableTasks from '$lib/components/gym/AvailableTasks.svelte';
  import { Wallet, History, Zap } from 'lucide-svelte';
  import WalletButton from '$lib/components/WalletButton.svelte';

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
  let pendingRecordings: any[] = [];

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
      viralBalance = await getBalance();
      recentSubmissions = await listSubmissions();
      earnedThisMonth = calculateEarnedRewards(recentSubmissions);
    } catch (error) {
      console.error('Failed to load balance:', error);
    }
  }

  async function loadUnclaimedRewards() {
    try {
      const recordings = await invoke<any[]>('list_recordings');
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

<div class="h-full max-w-7xl mx-auto overflow-x-hidden">
  {#if poolName}
    <div class="mx-auto mb-8">
      <div class="text-xl font-semibold mb-2">Viewing Forge: {poolName}</div>
    </div>
  {:else}
    <div class="mx-auto mb-8">
      <p class="text-gray-600">
        Choose a task, record a demonstration on your desktop, earn rewards. Your data helps us
        build the largest open-source dataset for training sophisticated AI assistants.
      </p>
    </div>
  {/if}
  {#if !$walletAddress}
    <div class="bg-primary-400 rounded-md p-5 flex flex-row text-white mb-6 -mt-2">
      <h1 class="grow justify-start align-middle">Please connect your wallet to use the gym.</h1>
      <div class="justify-end gorw-0">
        <WalletButton variant="large" />
      </div>
    </div>
  {/if}

  <div class={`${!$walletAddress ? 'blur-sm pointer-events-none' : ''}`}>
    <div class={`grid grid-cols-1 md:grid-cols-3 gap-4 mb-6 `}>
      <!-- Wallet Balance Card -->
      <Card padding="none" className="relative overflow-hidden border-0 shadow-md h-full">
        <div
          class="bg-gradient-to-br from-secondary-500 to-secondary-200 text-white h-full flex flex-col">
          <div class="p-4 flex-grow">
            <div class="flex items-center gap-2 mb-2">
              <Wallet size={14} />
              <span class="text-xs font-medium">Your Balance</span>
            </div>
            <div class="flex items-baseline gap-2 mb-3 flex-wrap">
              <div class="text-2xl font-bold">{formatNumber(viralBalance + unclaimedRewards)}</div>
              <div class="text-xs font-medium opacity-75">$VIRAL</div>
            </div>

            <div class="flex items-center gap-2 mt-4">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round">
                <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline>
                <polyline points="17 6 23 6 23 12"></polyline>
              </svg>
              <span class="text-xs font-medium">Earned This Month</span>
            </div>
            <div class="text-lg font-semibold mt-1">
              {formatNumber(earnedThisMonth)}
              <span class="text-xs font-medium opacity-75">$VIRAL</span>
            </div>

            <div class="flex items-center gap-2 mt-3">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="w-3.5 h-3.5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round">
                <path
                  d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3">
                </path>
              </svg>
              <span class="text-xs font-medium">Average Per Task</span>
            </div>
            <div class="text-base font-semibold mt-1">
              {formatNumber(averageEarnings)}
              <span class="text-xs font-medium opacity-75">$VIRAL</span>
            </div>
          </div>

          <div class="flex border-t border-white/20 text-xs mt-auto">
            <a
              href="/app/gym/skills"
              class="flex-1 py-3 flex items-center justify-center gap-1.5 font-medium hover:bg-white/10 transition-colors">
              <Zap size={12} />
              <span>Skill Tree</span>
            </a>
            <div class="w-px bg-white/20"></div>
            <a
              href="/app/gym/history"
              class="flex-1 py-3 flex items-center justify-center gap-1.5 font-medium hover:bg-white/10 transition-colors">
              <History size={12} />
              <span>History</span>
            </a>
          </div>
        </div>
      </Card>

      <!-- Pending Uploads Card - Gamified Version -->
      <Card
        padding="none"
        className="relative overflow-hidden border-0 shadow-md col-span-2 h-full">
        <div class="bg-gradient-to-br from-gray-700 to-gray-900 text-white h-full flex flex-col">
          <div class="p-4 flex flex-col h-full">
            {#if pendingRecordings.length > 0}
              <div class="mb-4">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="w-5 h-5 text-yellow-300"
                      viewBox="0 0 24 24"
                      fill="currentColor">
                      <path
                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm.31-8.86c-1.77-.45-2.34-.94-2.34-1.67 0-.84.79-1.43 2.1-1.43 1.38 0 1.9.66 1.94 1.64h1.71c-.05-1.34-.87-2.57-2.49-2.97V5H10.9v1.69c-1.51.32-2.72 1.3-2.72 2.81 0 1.79 1.49 2.69 3.66 3.21 1.95.46 2.34 1.15 2.34 1.87 0 .53-.39 1.39-2.1 1.39-1.6 0-2.23-.72-2.32-1.64H8.04c.1 1.7 1.36 2.66 2.86 2.97V19h2.34v-1.67c1.52-.29 2.72-1.16 2.73-2.77-.01-2.2-1.9-2.96-3.66-3.42z" />
                    </svg>
                    <h3 class="text-lg font-bold">You have pending rewards!</h3>
                  </div>

                  <div class="flex items-baseline gap-1.5">
                    <div class="text-lg font-bold text-yellow-300">
                      +{formatNumber(unclaimedRewards)}
                    </div>
                    <div class="text-xs font-medium opacity-75">$VIRAL</div>
                  </div>
                </div>
              </div>

              <!-- Gamified Bubble Grid -->
              <div class="flex-grow">
                <div class="grid grid-cols-5 sm:grid-cols-6 gap-1.5">
                  {#each pendingRecordings.slice(0, 8) as recording, i}
                    <a href="/app/gym/history" class="group">
                      <div
                        class="aspect-square rounded-xl opacity-75 hover:opacity-100
                      {i % 4 === 0
                          ? 'bg-gradient-to-br from-pink-500/40 to-red-500/40'
                          : i % 4 === 1
                            ? 'bg-gradient-to-br from-blue-500/40 to-cyan-500/40'
                            : i % 4 === 2
                              ? 'bg-gradient-to-br from-green-500/40 to-emerald-500/40'
                              : 'bg-gradient-to-br from-amber-500/40 to-orange-500/40'} 
                      p-1 flex flex-col items-center justify-center text-center transform hover:scale-105 transition-all
                      {i % 4 === 0
                          ? 'hover:shadow-lg hover:shadow-pink-500/30'
                          : i % 4 === 1
                            ? 'hover:shadow-lg hover:shadow-blue-500/30'
                            : i % 4 === 2
                              ? 'hover:shadow-lg hover:shadow-green-500/30'
                              : 'hover:shadow-lg hover:shadow-amber-500/30'} 
                      cursor-pointer">
                        <div class="relative">
                          <div
                            class="w-10 h-10 rounded-full
                          {i % 4 === 0
                              ? 'bg-gradient-to-br from-pink-400 to-red-500'
                              : i % 4 === 1
                                ? 'bg-gradient-to-br from-blue-400 to-cyan-500'
                                : i % 4 === 2
                                  ? 'bg-gradient-to-br from-green-400 to-emerald-500'
                                  : 'bg-gradient-to-br from-amber-400 to-orange-500'} 
                          flex items-center justify-center mb-1 group-hover:animate-pulse overflow-hidden">
                            {#if recording.quest?.icon_url}
                              <img
                                src={recording.quest.icon_url}
                                alt={recording.quest.app || 'App'}
                                class="w-6 h-6 object-contain" />
                            {:else}
                              <span class="text-white text-sm font-bold">
                                {formatNumber(recording.quest?.reward?.max_reward || 0).split(
                                  '.'
                                )[0]}
                              </span>
                            {/if}
                          </div>
                        </div>
                        <div class="text-xs font-medium text-white/90 mt-1 px-1 truncate w-full">
                          {recording.quest?.title?.split(' ').slice(0, 2).join(' ') || 'Recording'}
                        </div>
                        <div class="text-xs font-extrabold text-white/50 px-1 truncate w-full">
                          {formatNumber(recording.quest?.reward?.max_reward || 0).split('.')[0]} $VIRAL
                        </div>
                      </div>
                    </a>
                  {/each}
                </div>

                {#if pendingRecordings.length > 8}
                  <div class="text-center mt-3 text-sm font-medium text-white/80">
                    +{pendingRecordings.length - 8} more bubbles to pop!
                  </div>
                {/if}
              </div>

              <div class="mt-auto pt-4">
                <Button
                  href="/app/gym/history"
                  variant="primary"
                  behavior="none"
                  class="
                w-full justify-center bg-white text-gray-800!
                hover:bg-gray-200! border-0 py-2.5 text-sm font-bold shadow-lg rounded-full">
                  Claim Rewards Now
                </Button>
              </div>
            {:else}
              <div>
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="w-5 h-5 text-yellow-300"
                      viewBox="0 0 24 24"
                      fill="currentColor">
                      <path
                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm.31-8.86c-1.77-.45-2.34-.94-2.34-1.67 0-.84.79-1.43 2.1-1.43 1.38 0 1.9.66 1.94 1.64h1.71c-.05-1.34-.87-2.57-2.49-2.97V5H10.9v1.69c-1.51.32-2.72 1.3-2.72 2.81 0 1.79 1.49 2.69 3.66 3.21 1.95.46 2.34 1.15 2.34 1.87 0 .53-.39 1.39-2.1 1.39-1.6 0-2.23-.72-2.32-1.64H8.04c.1 1.7 1.36 2.66 2.86 2.97V19h2.34v-1.67c1.52-.29 2.72-1.16 2.73-2.77-.01-2.2-1.9-2.96-3.66-3.42z" />
                    </svg>
                    <h3 class="text-lg font-bold">Payouts</h3>
                  </div>
                </div>
              </div>
              <!-- Empty State -->
              <div
                class="flex flex-col items-center justify-center p-1 text-center gap-2 pb-3 opacity-75">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="w-16 h-16 text-indigo-300"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1"
                  stroke-linecap="round"
                  stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                  <path d="M12 8v4"></path>
                  <path d="M12 16h.01"></path>
                </svg>
                <h3 class="text-xl font-bold">No Pending Rewards</h3>
                <p class="text-indigo-200 max-w-xs mx-auto">
                  Why not start some tasks and earn some sweet $VIRAL tokens?
                </p>
              </div>
            {/if}
          </div>
        </div>
      </Card>
    </div>

    <AvailableTasks {poolId} {loadingApps} isGymBuilder={false} />
  </div>
</div>
