<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from './Card.svelte';
  import CreatePoolModal from './CreatePoolModal.svelte';
  import { ChevronDown, ChevronRight, Train, TrainIcon, RefreshCw, AlertTriangle, DollarSign, Settings } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { walletAddress } from '$lib/stores/wallet';
  import { listPools, createPool, updatePool, refreshPool } from '$lib/api/forge';
  import { type TrainingPool, type Token, TrainingPoolStatus } from '$lib/types/forge';
  import Button from './Button.svelte';
  import CopyButton from './CopyButton.svelte';
  import TextArea from './TextArea.svelte';
  import Input from './Input.svelte';

  let trainingPools: TrainingPool[] = [];
  let showCreateModal = false;
  let loading = true;
  let error: string | null = null;
  let refreshingPools: Set<string> = new Set();
  
  // Pricing data
  let viralPrice = 0;
  let solPrice = 0;
  let viralPerSol = 0;
  let loadingPrices = true;

  // Token contract addresses
  const TOKEN_DATA = {
    contractAddress: 'HW7D5MyYG4Dz2C98axfjVBeLWpsEnofrqy6ZUwqwpump', // VIRAL token
    solAddress: 'So11111111111111111111111111111111111111112' // SOL token
  };

  onMount(() => {
    fetchPrices();
  });

  async function fetchPrices() {
    try {
      loadingPrices = true;
      const response = await fetch(
        `https://api.jup.ag/price/v2?ids=${TOKEN_DATA.contractAddress},${TOKEN_DATA.solAddress}`
      );
      const json = await response.json();

      viralPrice = parseFloat(json.data[TOKEN_DATA.contractAddress].price);
      solPrice = parseFloat(json.data[TOKEN_DATA.solAddress].price);
      viralPerSol = solPrice / viralPrice;
    } catch (error) {
      console.error('Error fetching prices:', error);
    } finally {
      loadingPrices = false;
    }
  }

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
    updates: { status?: TrainingPoolStatus.live | TrainingPoolStatus.paused; skills?: string; pricePerDemo?: number }
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

  function handlePriceChange(pool: TrainingPool, price: number) {
    // Ensure price is at least 1 VIRAL
    pool.pricePerDemo = Math.max(1, price);
    pool.unsavedPrice = true;
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

  async function handleSavePrice(pool: TrainingPool) {
    try {
      await handleUpdatePool(pool, { pricePerDemo: pool.pricePerDemo });
      pool.unsavedPrice = false;
      trainingPools = trainingPools;
    } catch (err) {
      // Error is already handled by handleUpdatePool
    }
  }

  function setRecommendedPrice(pool: TrainingPool) {
    handlePriceChange(pool, 7); // Set recommended price to 7 VIRAL
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
      Train AI agents with crowd-powered demonstrations
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
            onkeydown={() => togglePool(pool)}
            onclick={() => togglePool(pool)}>
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
                  {:else if pool.status === TrainingPoolStatus.noGas}
                    no gas
                  {:else}
                    live
                  {/if}
                </div>
              </div>
              <div class="text-sm text-gray-500 flex items-center gap-3">
                <span>{pool.demonstrations.toLocaleString()} demos</span>
                <span class="text-gray-300">·</span>
                <span>{pool.pricePerDemo || 1} <b>{pool.token.symbol}</b>/demo</span>
                <span class="text-gray-300">·</span>
                <span>{pool.funds.toLocaleString()} <b>{pool.token.symbol}</b> balance</span>
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
            {#if pool.status === TrainingPoolStatus.noGas || pool.status === TrainingPoolStatus.noFunds}
            <div class="p-4 my-3 bg-amber-50 border-l-4 border-amber-400 rounded-lg flex gap-3 shadow-sm">
              <div class="text-amber-500 flex-shrink-0">
                <AlertTriangle size={18} />
              </div>
              <div class="flex-1">
                {#if pool.status === TrainingPoolStatus.noGas}
                  <p class="text-sm text-amber-900 font-semibold mb-1">
                    Missing SOL for Gas
                  </p>
                  <p class="text-xs leading-relaxed text-amber-800">
                    Your gym needs SOL to pay for on-chain transactions. Without gas, the gym cannot function on the Solana blockchain.
                  </p>
                {:else if pool.status === TrainingPoolStatus.noFunds}
                  <p class="text-sm text-amber-900 font-semibold mb-1">
                    Missing VIRAL Tokens
                  </p>
                  <p class="text-xs leading-relaxed text-amber-800">
                    Your gym needs VIRAL tokens to reward users who provide demonstrations. Without funds, users won't receive compensation.
                  </p>
                {/if}
                <p class="text-xs text-amber-800 font-medium mt-2">
                  Deposit {pool.status === TrainingPoolStatus.noGas ? 'SOL' : 'VIRAL'} to the address above to activate your gym and start collecting data.
                </p>
              </div>
            </div>
          {/if}

            <div class="mt-4 space-y-4">
              <div>
                <div class="flex justify-between items-center mb-2">
                  <div>
                    <p class="text-sm font-semibold">Skills to Train</p>
                    <p class="text-xs text-gray-500 italic">
                      Customize your AI agent gym's training focus. These skills generate new tasks for user demonstrations.
                    </p>
                  </div>
                  <div class="flex gap-2">
                    {#if pool.unsavedSkills}
                      <Button
                        class="px-3 py-1.5 text-sm border-green-500! text-green-600! hover:bg-green-100! bg-gray-200!"
                        onclick={() => handleSaveSkills(pool)}>
                        Save Skills
                      </Button>
                    {/if}
                    {#if pool.status !== TrainingPoolStatus.noFunds && pool.status !== TrainingPoolStatus.noGas}
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

              <div>
                <div class="flex justify-between items-center mb-2">
                  <div>
                    <div class="flex items-center gap-2">
                      <p class="text-sm font-semibold">Pricing Settings</p>
                    </div>
                    <p class="text-xs text-gray-500 italic">
                      Set compensation rates for users who contribute demonstration data to your gym.
                    </p>
                  </div>
                  <div class="flex gap-2">
                    {#if pool.unsavedPrice}
                      <Button
                        class="px-3 py-1.5 text-sm border-green-500! text-green-600! hover:bg-green-100! bg-gray-200!"
                        onclick={() => handleSavePrice(pool)}>
                        Save Price
                      </Button>
                    {/if}
                  </div>
                </div>
                
                <div class="bg-gray-100 p-4 rounded-lg">
                  <div class="mb-3">
                    <label class="flex items-center justify-between mb-2">
                      <span class="text-sm font-medium">Price per demonstration</span>
                      <Button 
                        class="px-2 py-1 text-xs bg-blue-50! text-blue-600! border-blue-200!"
                        onclick={() => setRecommendedPrice(pool)}>
                        Use recommended (7 VIRAL)
                      </Button>
                    </label>
                    <div class="flex items-center gap-2">
                      <input
                        type="number"
                        min="1"
                        step="0.1"
                        class="flex-1 p-2 bg-white border border-gray-300 rounded-lg"
                        value={pool.pricePerDemo || 1}
                        oninput={(e) => handlePriceChange(pool, parseFloat(e.currentTarget.value))} />
                      <span class="text-sm font-medium">{pool.token.symbol}</span>
                    </div>
                    <p class="text-xs text-gray-500 mt-1">
                      Minimum price: 1 {pool.token.symbol}
                    </p>
                  </div>

                  {#if !loadingPrices && viralPrice > 0}
                    <div class="bg-gradient-to-r from-blue-50 to-indigo-50 p-4 rounded-lg border border-blue-200 shadow-sm mt-4">
                      <div class="flex items-center gap-2 mb-3">
                        <DollarSign size={16} class="text-blue-600" />
                        <p class="text-sm font-bold text-blue-800">Pricing Calculator</p>
                      </div>
                      
                      <div class="grid grid-cols-2 gap-3">
                        <div class="bg-white/60 p-2 rounded border border-blue-100">
                          <p class="text-xs text-blue-500 mb-1">VIRAL Market Price</p>
                          <p class="text-sm font-bold text-blue-900">${viralPrice.toFixed(4)} <span class="text-xs font-normal">USD per token</span></p>
                        </div>
                        
                        <div class="bg-white/60 p-2 rounded border border-blue-100">
                          <p class="text-xs text-blue-500 mb-1">Reward in USD</p>
                          <p class="text-sm font-bold text-blue-900">${((pool.pricePerDemo || 1) * viralPrice).toFixed(2)} <span class="text-xs font-normal">per demo</span></p>
                        </div>
                      </div>
                      
                      <div class="mt-3 bg-blue-100/50 p-3 rounded-lg border border-blue-200">
                        <p class="text-sm text-blue-800 font-medium mb-1">Demonstration Capacity</p>
                        <div class="flex flex-col gap-3">
                          <div class="flex items-center justify-center gap-2 py-1">
                            <div class="flex flex-col items-center">
                              <div class="bg-white px-3 py-1 rounded-lg border border-blue-200 font-bold text-blue-900">
                                {pool.funds.toLocaleString()} {pool.token.symbol}
                              </div>
                              <p class="text-xs text-blue-600 mt-1">Pool Balance</p>
                            </div>
                            <div class="text-blue-500 font-bold">÷</div>
                            <div class="flex flex-col items-center">
                              <div class="bg-white px-3 py-1 rounded-lg border border-blue-200 font-bold text-blue-900">
                                {(pool.pricePerDemo || 1).toLocaleString()} {pool.token.symbol}
                              </div>
                              <p class="text-xs text-blue-600 mt-1">Price Per Demo</p>
                            </div>
                            <div class="text-blue-500 font-bold">=</div>
                            <div class="flex flex-col items-center">
                              <div class="bg-indigo-100 px-3 py-1 rounded-lg border border-blue-300 font-bold text-indigo-900">
                                {Math.floor(pool.funds / (pool.pricePerDemo || 1)).toLocaleString()}
                              </div>
                              <p class="text-xs text-blue-600 mt-1">Possible Demos</p>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  {/if}
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
