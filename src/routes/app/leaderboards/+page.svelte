<script lang="ts">
  import { getLeaderboardData } from '$lib/api/endpoints/gym/leaderboards';
  import Card from '$lib/components/Card.svelte';
  import NicknameModal from '$lib/components/modals/NicknameModal.svelte';
  import { nickname } from '$lib/stores';
  import { Users, Zap, Hammer, DollarSign, Check, LoaderCircle } from 'lucide-svelte';
  import { onMount } from 'svelte';

  // State management
  let loading = $state(true);
  let openNickModal = $state(false);
  let activeTab = $state('workers');
  let totalWorkers = $state(0);
  let tasksCompleted = $state(0);
  let totalRewards = $state(0);
  let activeForges = $state(0);

  // Mock data for the leaderboard with SOL addresses and some nicknames (reduced)
  let topWorkers: {
    rank: number;
    address: string;
    nickname?: string;
    tasks: number;
    rewards: number;
  }[] = $state([]);

  let topForges: {
    rank: number;
    name: string;
    tasks: number;
    payout: number;
  }[] = $state([]);

  onMount(async () => {
    const res = await getLeaderboardData();
    topWorkers = res.workersLeaderboard;
    topForges = res.forgeLeaderboard;
    totalWorkers = res.stats.totalWorkers;
    totalRewards = res.stats.totalRewards;
    tasksCompleted = res.stats.tasksCompleted;
    activeForges = res.stats.activeForges;
    loading = false;
  });

  // Leaderboard data to display based on active tab and view mode
  const leaderboardData = $derived(
    activeTab === 'workers'
      ? topWorkers.map((worker) => ({
          ...worker
        }))
      : topForges.map((forge) => ({
          ...forge
        }))
  );

  // Helper functions
  const formatCurrency = (value: number) => {
    return value.toFixed(2);
  };

  const truncateAddress = (address: string) => {
    if (!address) return '';
    return address.slice(0, 6) + '...' + address.slice(-6);
  };
</script>

<div class="p-2 sm:p-4">
  <div class="max-w-7xl mx-auto mb-6">
    <div class="flex justify-between items-center">
      <h2 class="text-2xl font-title text-blac">Leaderboards</h2>
    </div>

    <!-- Tab Navigation -->
    <div class="border-b border-accent-400 mt-5">
      <div class="flex -mb-px">
        <button
          class="py-3 px-6 font-title flex gap-2 transition border-b-2 {activeTab === 'workers'
            ? 'border-secondary-300 text-secondary-300'
            : 'border-transparent text-accent-600 hover:text-primary-300'}"
          onclick={() => (activeTab = 'workers')}>
          <Users class="w-5 h-5" />
          Top Demonstrators
        </button>
        <button
          class="py-3 px-6 font-title flex gap-2 border-b-2 {activeTab === 'forges'
            ? 'border-secondary-300 text-secondary-300'
            : 'border-transparent text-accent-600 hover:text-primary-300'}"
          onclick={() => (activeTab = 'forges')}>
          <Hammer class="w-5 h-5" />
          Top Forges
        </button>
      </div>
    </div>

    {#if !loading}
      <div class="mb-8 mt-2">
        <!-- Leaderboard Table -->
        <Card padding="none">
          <div class="grid grid-cols-12 px-6 py-4 bg-accent-200 text-accent-600 text-sm">
            <div class="col-span-1">Rank</div>
            <div class="col-span-6">
              {activeTab === 'workers' ? 'Address' : 'Name'}
            </div>
            <div class="col-span-5 grid grid-cols-2 text-right">
              <div class="col-span-1">
                {#if activeTab === 'workers'}
                  Tasks Completed
                {:else}
                  Tasks Created
                {/if}
              </div>
              <div class="col-span-1">
                {#if activeTab === 'workers'}
                  $VIRAL Rewards Earned
                {:else}
                  $VIRAL Paid Out
                {/if}
              </div>
            </div>
          </div>

          <div class="max-h-[50vh] overflow-y-auto">
            {#each leaderboardData as item, index}
              <div
                class="grid grid-cols-12 px-6 py-4 {index !== leaderboardData.length - 1
                  ? 'border-b border-accent-400'
                  : ''} {index % 2 === 0 ? 'bg-accent-100' : 'bg-accent-200'}">
                <div class="col-span-1">
                  <div
                    class="w-8 h-8 rounded-full flex items-center justify-center {item.rank <= 3
                      ? 'text-accent-100 ' +
                        (item.rank === 1
                          ? 'bg-yellow-500'
                          : item.rank === 2
                            ? 'bg-accent-500'
                            : 'bg-amber-700')
                      : 'bg-accent-400 text-primary-300'}">
                    {item.rank}
                  </div>
                </div>

                <!-- Address/Name column -->
                <div class="col-span-6 flex items-center">
                  {#if activeTab === 'workers' && 'address' in item}
                    <div class="flex flex-col">
                      <span class="font-medium text-primary-100 font-mono text-xs">
                        {truncateAddress(item.address)}
                      </span>
                      {#if item.nickname}
                        <span class="text-xs text-secondary-300">
                          {item.nickname}
                        </span>
                      {/if}
                    </div>
                  {:else if 'name' in item}
                    <span class="font-medium text-primary-100">{item.name}</span>
                  {/if}
                </div>

                <!-- Value column -->
                <div class="col-span-5 grid grid-cols-2 text-right">
                  <div class="col-span-1 flex items-center justify-end">
                    <div class="flex items-center">
                      <span
                        class="text-sm font-semibold bg-secondary-600/30 px-2 py-0.5 rounded-lg text-secondary-600">
                        {item.tasks}
                        Tasks
                      </span>
                    </div>
                  </div>
                  <div class="col-span-1 flex items-center justify-end">
                    <div class="flex items-center">
                      <span
                        class="text-sm bg-secondary-300/30 font-semibold bg-opacity-20 px-2 py-0.5 rounded-lg text-secondary-300">
                        {#if 'address' in item}
                          {formatCurrency(item.rewards)} $VIRAL
                        {:else}
                          {formatCurrency(item.payout)} $VIRAL
                        {/if}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </Card>
        <button
          onclick={() => (openNickModal = true)}
          class="text-gray-600 ml-2 text-xs italic hover:underline transition-all">
          Set your leaderboard nickname.
        </button>
      </div>

      <!-- Global Stats Section -->
      <div class="mt-12">
        <h2 class="text-xl font-title text-primary-100 mb-6 flex items-center">
          <Zap class="text-secondary-100 mr-2" size={20} />
          Global Statistics
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <Card>
            <div class="flex justify-between items-center mb-2">
              <span class="text-primary-300 text-sm">Total Demonstrators</span>
              <Users size={16} class="text-secondary-600" />
            </div>
            <div class="text-2xl font-title text-primary-100">{totalWorkers}</div>
          </Card>
          <Card>
            <div class="flex justify-between items-center mb-2">
              <span class="text-primary-300 text-sm">Total Tasks Completed</span>
              <Check size={16} class="text-secondary-600" />
            </div>
            <div class="text-2xl font-title text-primary-100 flex items-center">
              {tasksCompleted}
            </div>
          </Card>
          <Card>
            <div class="flex justify-between items-center mb-2">
              <span class="text-primary-300 text-sm">Total Paid Out</span>
              <DollarSign size={16} class="text-secondary-600" />
            </div>
            <div class="text-2xl font-title text-primary-100 flex items-center">
              {formatCurrency(totalRewards)} $VIRAL
            </div>
          </Card>
          <Card>
            <div class="flex justify-between items-center mb-2">
              <span class="text-primary-300 text-sm">Active Forges</span>
              <Hammer size={16} class="text-secondary-600" />
            </div>
            <div class="text-2xl font-title text-primary-100">{activeForges}</div>
          </Card>
        </div>
      </div>
    {:else}
      <div class="mx-auto w-full mt-10">
        <LoaderCircle class="animate-spin mx-auto text-secondary-300 w-16 h-16" />
      </div>
    {/if}
  </div>
</div>

<NicknameModal bind:open={openNickModal} oldNick={$nickname} />
