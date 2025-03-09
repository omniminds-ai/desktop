<script lang="ts">
  import Input from '../Input.svelte';
  import Button from '../Button.svelte';
  import CopyButton from '../CopyButton.svelte';
  import Card from '../Card.svelte';
  import { RefreshCw, Play, Pause } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
  };
  export let onSave: (pool: TrainingPool, updates: any) => Promise<void>;
  export let onRefresh: (poolId: string) => Promise<void>;
  export let refreshingPools: Set<string>;
  export let unsavedChanges: boolean;
  export let handleSaveChanges: () => Promise<void>;

  function handleNameChange(event: Event) {
    pool.name = (event.target as HTMLInputElement).value;
    pool.unsavedName = true;
    unsavedChanges = true;
  }

  function handlePriceChange(event: Event) {
    const price = parseFloat((event.target as HTMLInputElement).value);
    // Ensure price is at least 1 VIRAL
    pool.pricePerDemo = Math.max(1, price);
    pool.unsavedPrice = true;
    unsavedChanges = true;
  }

  function formatNumber(num: number) {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
</script>

<div class="space-y-6">
  <Card
    padding="md"
    className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1">Gym Name</label>
        <Input
          type="text"
          variant="light"
          value={pool.name}
          oninput={handleNameChange}
          placeholder="Enter gym name" />
      </div>

      <div>
        <div class="flex justify-between items-center mb-1">
          <label class="block text-sm font-medium">Deposit Address ({pool.token.symbol})</label>
          <Button
            class="px-2 py-1 text-xs"
            variant="secondary"
            onclick={() => onRefresh(pool._id)}
            disabled={refreshingPools.has(pool._id)}>
            <RefreshCw size={12} class={refreshingPools.has(pool._id) ? 'animate-spin' : ''} />
          </Button>
        </div>
        <div class="flex gap-2 items-center">
          <input
            type="text"
            class="flex-1 p-2 bg-gray-100 rounded-lg cursor-text text-sm"
            readonly
            value={pool.depositAddress} />
          <CopyButton content={pool.depositAddress} />
        </div>
        <p class="text-xs text-gray-500 mt-1">
          Send {pool.token.symbol} tokens to this address to fund your gym.
        </p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1">Price per Demonstration</label>
        <div class="flex items-center gap-2">
          <input
            type="number"
            min="1"
            step="0.1"
            class="flex-1 p-2 bg-gray-100 rounded-lg border-0"
            value={pool.pricePerDemo || 1}
            oninput={handlePriceChange} />
          <span class="text-sm font-medium">{pool.token.symbol}</span>
        </div>
        <p class="text-xs text-gray-500 mt-1">
          Minimum price: 1 {pool.token.symbol}
        </p>
      </div>

      <div class="p-3 bg-blue-50 rounded-lg border border-blue-100">
        <div class="flex justify-between mb-1">
          <span class="text-sm text-blue-800 font-medium">Current Balance:</span>
          <span class="text-sm font-bold text-blue-800">
            {formatNumber(pool.funds)}
            {pool.token.symbol}
          </span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-blue-800 font-medium">Possible Demos:</span>
          <span class="text-sm font-bold text-blue-800">
            {Math.floor(pool.funds / (pool.pricePerDemo || 1))}
          </span>
        </div>
      </div>
    </div>
  </Card>

  <div class="flex flex-col gap-2">
    {#if unsavedChanges}
      <Button
        class="w-full justify-center bg-green-500! text-white! hover:bg-green-600!"
        onclick={handleSaveChanges}>
        Save Changes
      </Button>
    {/if}

    <Button
      behavior="none"
      class="py-4!"
      variant={pool.status === TrainingPoolStatus.live ? 'secondary' : 'green'}
      title={pool.status === TrainingPoolStatus.live ? 'Pause Gym' : 'Activate Gym'}
      onclick={() =>
        onSave(pool, {
          status:
            pool.status === TrainingPoolStatus.live
              ? TrainingPoolStatus.paused
              : TrainingPoolStatus.live
        })}>
      <div class="flex items-center">
        {#if pool.status === TrainingPoolStatus.live}
          <Pause size={16} class="mr-2" />
          Pause Gym
        {:else}
          <Play size={16} class="mr-2" />
          Activate Gym
        {/if}
      </div>
    </Button>
  </div>
</div>
