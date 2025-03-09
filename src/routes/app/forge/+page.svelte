<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import CreatePoolModal from '$lib/components/CreatePoolModal.svelte';
  import GenerateGymModal from '$lib/components/GenerateGymModal.svelte';
  import { ChevronRight } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import { walletAddress } from '$lib/stores/wallet';
  import {
    listPools,
    createPool,
    updatePool,
    refreshPool,
    generateApps,
    createPoolWithApps
  } from '$lib/api/forge';
  import { type TrainingPool, type Token, TrainingPoolStatus } from '$lib/types/forge';
  import WalletButton from '$lib/components/WalletButton.svelte';

  // Extended TrainingPool for UI state
  type ExtendedPool = TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
    tokenBalance?: number;
    solBalance?: number;
  };

  let trainingPools: ExtendedPool[] = [];
  let showCreateModal = false;
  let showGenerateGymModal = false;
  let currentSkills = '';
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
      let pools = await listPools($walletAddress);
      // Initialize UI state properties
      trainingPools = pools.map((pool) => ({
        ...pool,
        unsavedSkills: false,
        unsavedPrice: false,
        unsavedName: false
      }));
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load AI agent gyms';
    } finally {
      loading = false;
    }
  }

  let refreshIntervals: { [key: string]: number } = {};

  // Cleanup intervals on component destroy
  onDestroy(() => {
    Object.values(refreshIntervals).forEach((interval) => {
      clearInterval(interval);
    });
  });

  function navigateToGymDetail(pool: ExtendedPool) {
    // Navigate to the gym detail page using querystring
    goto(`/app/forge/gym?id=${pool._id}`);
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
      // Try to generate apps first if we have skills
      let apps = undefined;
      if (data.skills.trim()) {
        try {
          const response = await generateApps(data.skills);
          if (response?.content?.apps?.length > 0) {
            apps = response.content.apps;
          }
        } catch (genError) {
          console.error('Failed to generate apps:', genError);
          // Continue without apps
        }
      }

      // Create pool with apps if available
      if (apps) {
        await createPoolWithApps({
          ...data,
          apps,
          ownerAddress: $walletAddress
        });
      } else {
        await createPool({
          ...data,
          ownerAddress: $walletAddress
        });
      }

      currentSkills = '';
      showCreateModal = false;
      loadPools();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to create AI agent gym';
    }
  }

  async function handleUpdatePool(pool: ExtendedPool, updates: any) {
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
</script>

<div class="h-full space-y-6 p-4">
  <div>
    <div class="flex justify-between items-center mb-2">
      <h2 class="text-2xl font-bold">Forge</h2>
    </div>
    <p class="text-gray-400">
      Collect crowd-powered demonstrations, perfect for training AI agents.
    </p>
  </div>

  {#if error}
    <div class="p-4 bg-red-500/10 text-red-500 rounded-lg">
      {error}
    </div>
  {/if}

  {#if !$walletAddress}
    <div class="bg-primary-400 rounded-xl p-10 mx-auto flex max-w-lg gap-10 flex-col text-white">
      <h1 class="align-middle text-center">Please connect your wallet to use the Forge.</h1>
      <div class="align-middle w-fit mx-auto">
        <WalletButton variant="large" />
      </div>
    </div>
  {:else if loading}
    <div class="text-center text-gray-400 py-8">Loading AI agent gyms...</div>
  {:else if trainingPools.length === 0}
    <div class="text-center text-gray-400 py-8">
      No AI agent gyms found. Create one to get started!
    </div>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <!-- Create New Gym Card -->
      <Card
        variant="secondary"
        padding="md"
        className="h-full flex flex-col border-2 border-dashed border-gray-300 bg-gray-50 hover:bg-gray-100 hover:border-secondary-300 transition-all duration-300">
        <div
          class="flex flex-col h-full items-center justify-center cursor-pointer"
          role="button"
          tabindex="0"
          onkeydown={() => (showGenerateGymModal = true)}
          onclick={() => (showGenerateGymModal = true)}>
          <div
            class="rounded-full bg-gray-200 w-24 h-24 flex items-center justify-center mb-4 duration-300 transform transition-all">
            <span class="text-5xl text-gray-500 font-light transition-colors duration-300">+</span>
          </div>
          <div
            class="text-lg font-medium text-gray-600 group-hover:text-gray-800 transition-colors duration-300">
            Create New Gym
          </div>
          <p
            class="text-sm text-gray-500 text-center mt-2 group-hover:text-gray-600 transition-colors duration-300">
            Start collecting demonstrations for your AI agent training
          </p>
        </div>
      </Card>

      {#each trainingPools as pool (pool._id)}
        <div
          class="cursor-pointer"
          role="button"
          tabindex="0"
          onkeydown={() => navigateToGymDetail(pool)}
          onclick={() => navigateToGymDetail(pool)}>
          <Card
            variant="secondary"
            padding="md"
            className="h-full flex flex-col hover:bg-white hover:shadow-lg! transition-shadow duration-300">
            <div class="flex flex-col h-full transition-opacity duration-300">
              <!-- Gym Name -->
              <div
                class="font-title text-lg mb-2 truncate {pool.status !== TrainingPoolStatus.live
                  ? 'opacity-50'
                  : ''}">
                {pool.name}
              </div>

              <!-- Status Badge -->
              <div class="flex items-center gap-1 mb-3">
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

              <!-- Stats -->
              <div class="mt-auto">
                <div class="flex items-center justify-between mt-2">
                  <div class="text-sm font-medium">
                    {(pool.tokenBalance || 0).toLocaleString()}
                    <span class="font-medium">{pool.token.symbol}</span>
                  </div>
                  <div class="text-sm text-gray-500">Pool Balance</div>
                </div>

                <!-- Possible demos calculation -->
                {#if pool.pricePerDemo && pool.pricePerDemo > 0}
                  {@const possibleDemos = Math.floor((pool.tokenBalance || 0) / pool.pricePerDemo)}
                  {@const totalDemos = pool.demonstrations + possibleDemos}
                  {@const demoPercentage =
                    totalDemos > 0 ? Math.min(100, (pool.demonstrations / totalDemos) * 100) : 0}

                  <div class="mt-2">
                    <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                      <span>Uploads / Remaining Demos</span>
                      <span>
                        {pool.demonstrations.toLocaleString()} / {possibleDemos.toLocaleString()}
                      </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-1.5">
                      <div
                        class="bg-gradient-to-br from-secondary-500 to-secondary-200 h-1.5 rounded-full"
                        style="width: {demoPercentage}%">
                      </div>
                    </div>
                  </div>
                {/if}

                <div class="flex justify-end mt-2">
                  <button
                    class="text-secondary-500 text-sm flex items-center gap-1 cursor-pointer hover:underline hover:text-blue-600 focus:outline-none"
                    onclick={(e) => {
                      e.stopPropagation();
                      navigateToGymDetail(pool);
                    }}>
                    View Details
                    <div class="text-gray-400">
                      <ChevronRight size={16} />
                    </div>
                  </button>
                </div>
              </div>
            </div>
          </Card>
        </div>
      {/each}
    </div>
  {/if}
</div>

<CreatePoolModal
  show={showCreateModal}
  onClose={() => (showCreateModal = false)}
  onCreate={handleCreatePool}
  initialSkills={currentSkills} />

<!-- Use GenerateGymModal for gym creation -->
<GenerateGymModal
  show={showGenerateGymModal}
  skills={currentSkills}
  on:change={(e) => (currentSkills = e.detail.skills)}
  on:close={() => (showGenerateGymModal = false)}
  on:save={async () => {
    try {
      // Try to generate apps based on the prompt first
      await generateApps(currentSkills);
    } catch (err) {
      console.error('Failed to pre-generate apps:', err);
    } finally {
      // Continue with creating the pool regardless of app generation success
      showGenerateGymModal = false;
      showCreateModal = true;
    }
  }} />
