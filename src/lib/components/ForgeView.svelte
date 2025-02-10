<script lang="ts">
  import Card from './Card.svelte';
  import CreatePoolModal from './CreatePoolModal.svelte';
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { walletAddress } from '$lib/stores/wallet';
  import { listPools, createPool, updatePool } from '$lib/api/forge';
  import type { TrainingPool, Token } from '$lib/types/forge';

  let trainingPools: TrainingPool[] = [];
  let showCreateModal = false;
  let loading = true;
  let error: string | null = null;

  $: if ($walletAddress) {
    loadPools();
  }

  async function loadPools() {
    if (!$walletAddress) return;

    try {
      loading = true;
      error = null;
      trainingPools = await listPools($walletAddress);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load training pools';
    } finally {
      loading = false;
    }
  }

  function previewQuest(skills: string) {
    goto(`/app/gym/chat?preview=${encodeURIComponent(skills)}`);
  }

  function togglePool(pool: TrainingPool) {
    pool.expanded = !pool.expanded;
    trainingPools = trainingPools;
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
      error = err instanceof Error ? err.message : 'Failed to create training pool';
    }
  }

  async function handleUpdatePool(
    pool: TrainingPool,
    updates: { status?: 'live' | 'paused'; skills?: string }
  ) {
    try {
      await updatePool({
        _id: pool._id,
        ...updates
      });
      loadPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to update training pool';
    }
  }

  function handleSkillsChange(pool: TrainingPool, skills: string) {
    handleUpdatePool(pool, { skills });
  }

  function handleStatusToggle(pool: TrainingPool) {
    handleUpdatePool(pool, {
      status: pool.status === 'live' ? 'paused' : 'live'
    });
  }
</script>

<div class="h-full space-y-6 p-4">
  <div>
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-2xl font-bold">Forge</h2>
      <button
        class="px-4 py-2 bg-secondary-300 text-white rounded-lg hover:bg-secondary-400 cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-secondary-300"
        on:click={() => (showCreateModal = true)}
        disabled={$walletAddress ? false : true}>
        New Training Pool
      </button>
    </div>
    <p class="text-gray-400">Turn expert skills into AI agents with crowd-powered training</p>
  </div>

  {#if error}
    <div class="p-4 bg-red-500/10 text-red-500 rounded-lg">
      {error}
    </div>
  {/if}

  {#if !$walletAddress}
    <div class="text-center text-gray-400 py-8">
      Connect your wallet to view and manage training pools
    </div>
  {:else if loading}
    <div class="text-center text-gray-400 py-8">Loading training pools...</div>
  {:else if trainingPools.length === 0}
    <div class="text-center text-gray-400 py-8">
      No training pools found. Create one to get started!
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
                  'live'
                    ? 'bg-green-500/10 text-green-500'
                    : pool.status === 'paused'
                      ? 'bg-gray-500/10 text-gray-500'
                      : 'bg-yellow-500/10 text-yellow-600'}">
                  <div
                    class="w-1.5 h-1.5 rounded-full {pool.status === 'live'
                      ? 'bg-green-500 animate-pulse'
                      : pool.status === 'paused'
                        ? 'bg-gray-500'
                        : 'bg-yellow-500 animate-pulse'}">
                  </div>
                  {pool.status}
                </div>
              </div>
              <div class="text-sm text-gray-500">
                {pool.demonstrations.toLocaleString()} demonstrations • {pool.funds.toLocaleString()}
                <b>{pool.token.symbol}</b>
              </div>
            </div>
            <div class="flex gap-2">
              <button
                class="px-3 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300 cursor-pointer transition-colors"
                on:click|stopPropagation={() => previewQuest(pool.skills)}>
                Preview Gym
              </button>
              <button
                class="px-3 py-1.5 bg-secondary-300 text-white text-sm rounded hover:bg-secondary-400 cursor-pointer transition-colors">
                Export Data
              </button>
              <div class="text-gray-400 cursor-pointer">
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
                  <div class="text-sm font-semibold">Skills to Collect</div>
                  {#if pool.status !== 'out of funds'}
                    <button
                      class="px-3 py-1.5 text-sm rounded border cursor-pointer transition-colors {pool.status ===
                      'live'
                        ? 'border-gray-300 text-gray-700 hover:bg-gray-200'
                        : 'border-green-500 text-green-600 hover:bg-green-100'}"
                      on:click={() => handleStatusToggle(pool)}>
                      {pool.status === 'live' ? 'Pause Pool' : 'Resume Pool'}
                    </button>
                  {/if}
                </div>
                <textarea
                  class="w-full h-32 p-3 bg-gray-100 rounded-lg resize-none hover:bg-gray-200 transition-colors"
                  placeholder="List the skills you want to collect demonstrations for..."
                  bind:value={pool.skills}
                  on:input={(e) => handleSkillsChange(pool, e.currentTarget.value)}>
                </textarea>
              </div>

              <div>
                <div class="text-sm font-semibold mb-2">Deposit Address ({pool.token.symbol})</div>
                <div class="flex gap-2 items-center">
                  <input
                    type="text"
                    class="flex-1 p-2 bg-gray-100 rounded-lg hover:bg-gray-200 cursor-text"
                    readonly
                    value={pool.depositAddress} />
                  <button
                    class="px-3 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300 cursor-pointer transition-colors"
                    on:click={() => navigator.clipboard.writeText(pool.depositAddress)}>
                    Copy
                  </button>
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
