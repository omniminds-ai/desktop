<script lang="ts">
  import { onDestroy } from 'svelte';
  import Card from './Card.svelte';
  import CreatePoolModal from './CreatePoolModal.svelte';
  import { ChevronDown, ChevronRight, Train, TrainIcon, RefreshCw } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { walletAddress } from '$lib/stores/wallet';
  import { listPools, createPool, updatePool, refreshPool } from '$lib/api/forge';
  import { type TrainingPool, type Token, TrainingPoolStatus } from '$lib/types/forge';
  import Button from './Button.svelte';
  import CopyButton from './CopyButton.svelte';
  import TextArea from './TextArea.svelte';

  let trainingPools: TrainingPool[] = [];
  let showCreateModal = false;
  let loading = true;
  let error: string | null = null;
  let refreshingPools: Set<string> = new Set();

  $: if ($walletAddress) {
    loadPools();
  }

  async function loadPools() {
    if (!$walletAddress) return;

    try {
      loading = true;
      error = null;
      trainingPools = await listPools($walletAddress);
      // Initialize unsavedSkills flag for each pool
      trainingPools = trainingPools.map((pool) => ({ ...pool, unsavedSkills: false }));
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load AI agent gyms';
    } finally {
      loading = false;
    }
  }

  function previewQuest(pool: TrainingPool) {
    goto(
      `/app/gym?poolId=${encodeURIComponent(pool._id)}&poolName=${encodeURIComponent(pool.name)}`
    );
  }

  let refreshIntervals: { [key: string]: number } = {};

  // Cleanup intervals on component destroy
  onDestroy(() => {
    Object.values(refreshIntervals).forEach((interval) => {
      clearInterval(interval);
    });
  });

  function togglePool(pool: TrainingPool) {
    pool.expanded = !pool.expanded;

    // Start periodic refresh if expanded
    if (pool.expanded) {
      // Initial refresh
      refreshPoolData(pool._id);
      // Set up interval for periodic refresh
      refreshIntervals[pool._id] = window.setInterval(() => {
        refreshPoolData(pool._id);
      }, 300000); // Refresh every 5 minutes
    } else {
      // Clear interval when collapsed
      if (refreshIntervals[pool._id]) {
        clearInterval(refreshIntervals[pool._id]);
        delete refreshIntervals[pool._id];
      }
    }

    trainingPools = trainingPools;
  }

  async function refreshPoolData(poolId: string) {
    if (refreshingPools.has(poolId)) return;

    try {
      refreshingPools.add(poolId);
      refreshingPools = refreshingPools; // Trigger reactivity

      // Create a promise that resolves after 1 second
      const minDelay = new Promise((resolve) => setTimeout(resolve, 1000));

      // Run the refresh and delay in parallel
      const [updatedPool] = await Promise.all([refreshPool(poolId), minDelay]);

      // Update the pool in the list
      trainingPools = trainingPools.map((pool) =>
        pool._id === poolId ? { ...pool, ...updatedPool, expanded: true } : pool
      );
    } catch (err) {
      console.error('Failed to refresh gym:', err);
    } finally {
      refreshingPools.delete(poolId);
      refreshingPools = refreshingPools; // Trigger reactivity
    }
  }

  async function handleCreatePool(data: { name: string; skills: string; token: Token }) {
    if (!$walletAddress) return;

    try {
      await createPool({
        ...data,
        ownerAddress: $walletAddress
      });
      showCreateModal = false;
      loadPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create AI agent gym';
    }
  }

  async function handleUpdatePool(
    pool: TrainingPool,
    updates: { status?: TrainingPoolStatus.live | TrainingPoolStatus.paused; skills?: string }
  ) {
    try {
      await updatePool({
        id: pool._id,
        ...updates
      });
      loadPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to update AI agent gym';
    }
  }

  function handleSkillsChange(pool: TrainingPool, skills: string) {
    pool.skills = skills;
    pool.unsavedSkills = true;
    trainingPools = trainingPools;
  }

  async function handleSaveSkills(pool: TrainingPool) {
    try {
      await handleUpdatePool(pool, { skills: pool.skills });
      pool.unsavedSkills = false;
      trainingPools = trainingPools;
    } catch (err) {
      // Error is already handled by handleUpdatePool
    }
  }

  function handleStatusToggle(pool: TrainingPool) {
    handleUpdatePool(pool, {
      status:
        pool.status === TrainingPoolStatus.live
          ? TrainingPoolStatus.paused
          : TrainingPoolStatus.live
    });
  }
</script>

<div class="h-full space-y-6 p-4">
  <div>
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-2xl font-bold">Forge</h2>
      <Button
        class="px-4! py-2!"
        onclick={() => (showCreateModal = true)}
        disabled={$walletAddress ? false : true}>
        Create New Gym
      </Button>
    </div>
    <p class="text-gray-400">
      Train AI agents with crowd-powered demonstrations in specialized skill gyms
    </p>
  </div>

  {#if error}
    <div class="p-4 bg-red-500/10 text-red-500 rounded-lg">
      {error}
    </div>
  {/if}

  {#if !$walletAddress}
    <div class="text-center text-blac py-8">
      Connect your wallet to view and manage your AI agent gyms
    </div>
  {:else if loading}
    <div class="text-center text-gray-400 py-8">Loading AI agent gyms...</div>
  {:else if trainingPools.length === 0}
    <div class="text-center text-gray-400 py-8">
      No AI agent gyms found. Create one to get started!
    </div>
  {:else}
    <div class="space-y-4">
      {#each trainingPools as pool (pool._id)}
        <Card variant="secondary" padding="md">
          <div
            class="flex justify-between items-center cursor-pointer"
            role="button"
            tabindex="0"
            on:keydown={() => togglePool(pool)}
            on:click={() => togglePool(pool)}>
            <div>
              <div class="flex items-center gap-2">
                <div class="font-title text-lg">{pool.name}</div>
                <div
                  class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full {pool.status ===
                  TrainingPoolStatus.live
                    ? 'bg-green-500/10 text-green-500'
                    : pool.status === TrainingPoolStatus.paused
                      ? 'bg-gray-500/10 text-gray-500'
                      : 'bg-yellow-500/10 text-yellow-600'}">
                  <div
                    class="w-1.5 h-1.5 rounded-full {pool.status === TrainingPoolStatus.live
                      ? 'bg-green-500 animate-pulse'
                      : pool.status === TrainingPoolStatus.paused
                        ? 'bg-gray-500'
                        : 'bg-yellow-500 animate-pulse'}">
                  </div>
                  {#if pool.status === TrainingPoolStatus.paused}
                    paused
                  {:else if pool.status === TrainingPoolStatus.noFunds}
                    no funds
                  {:else}
                    live
                  {/if}
                </div>
              </div>
              <div class="text-sm text-gray-500">
                {pool.demonstrations.toLocaleString()} demonstrations collected • {Math.floor(
                  pool.funds / 5.82
                ).toLocaleString()} remaining • {pool.funds.toLocaleString()}
                <b>{pool.token.symbol}</b>
              </div>
            </div>
            <div class="flex gap-2 items-center">
              <Button
                class="px-3 py-1.5 text-sm"
                variant="secondary"
                type="button"
                onclick={() => previewQuest(pool)}>
                View Gym
              </Button>
              <Button class="px-3 py-1.5 text-sm">Export Data</Button>
              <div class="text-gray-400 flex items-center cursor-pointer">
                {#if pool.expanded}
                  <ChevronDown size={24} />
                {:else}
                  <ChevronRight size={24} />
                {/if}
              </div>
            </div>
          </div>

          {#if pool.expanded}
            <div class="mt-4 space-y-4">
              <div>
                <div class="flex justify-between items-center mb-2">
                  <div class="text-sm font-semibold">Skills to Train</div>
                  <div class="flex gap-2">
                    {#if pool.unsavedSkills}
                      <Button
                        class="px-3 py-1.5 text-sm border-green-500! text-green-600! hover:bg-green-100! bg-gray-200!"
                        onclick={() => handleSaveSkills(pool)}>
                        Save Skills
                      </Button>
                    {/if}
                    {#if pool.status !== TrainingPoolStatus.noFunds}
                      <Button
                        class="px-3 py-1.5 text-sm bg-transparent! {pool.status ===
                        TrainingPoolStatus.live
                          ? 'border-gray-300! text-gray-700! hover:bg-gray-200!'
                          : 'border-green-500! text-green-600! hover:bg-green-100!'}"
                        onclick={() => handleStatusToggle(pool)}>
                        {pool.status === TrainingPoolStatus.live ? 'Pause Gym' : 'Resume Gym'}
                      </Button>
                    {/if}
                  </div>
                </div>
                <TextArea
                  class="h-32"
                  variant="light"
                  placeholder="List the skills you want to train your AI agent on (one per line)..."
                  bind:value={pool.skills}
                  oninput={(e) => handleSkillsChange(pool, e.currentTarget.value)}>
                </TextArea>
              </div>

              <div>
                <div class="flex justify-between items-center mb-2">
                  <div>
                    <p class="text-sm font-semibold">Deposit Address ({pool.token.symbol})</p>
                    <p class="text-xs text-gray-500 italic">
                      Send ${pool.token.symbol} to this address to fund your gym.
                    </p>
                  </div>
                  <Button
                    class="px-2 py-1 text-sm"
                    variant="secondary"
                    onclick={() => refreshPoolData(pool._id)}
                    disabled={refreshingPools.has(pool._id)}>
                    <span>Refresh</span>
                    <RefreshCw
                      size={14}
                      class={`inline mb-0.5 ${refreshingPools.has(pool._id) ? 'animate-spin' : ''}`} />
                  </Button>
                </div>
                <div class="flex gap-2 items-center">
                  <input
                    type="text"
                    class="flex-1 p-2 bg-gray-200 rounded-lg cursor-text"
                    readonly
                    value={pool.depositAddress} />
                  <CopyButton content={pool.depositAddress} />
                </div>
              </div>
            </div>
          {/if}
        </Card>
      {/each}
    </div>
  {/if}
</div>

<CreatePoolModal
  show={showCreateModal}
  onClose={() => (showCreateModal = false)}
  onCreate={handleCreatePool} />
