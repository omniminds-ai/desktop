<script lang="ts">
  import Card from '../Card.svelte';
  import CopyButton from '../CopyButton.svelte';
  import { AlertTriangle } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
  };
  
  function formatNumber(num: number) {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
</script>

<div class="space-y-6">
  <!-- Gym Overview Card -->
  <Card padding="md" className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-xl font-bold">{pool.name}</h3>
    </div>
    
    <!-- Key Stats -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      <div class="bg-blue-50 p-4 rounded-lg">
        <div class="text-sm text-blue-800">Total Demonstrations</div>
        <div class="text-2xl font-bold text-blue-900">{pool.demonstrations.toLocaleString()}</div>
      </div>
      
      <div class="bg-green-50 p-4 rounded-lg">
        <div class="text-sm text-green-800">Pool Balance</div>
        <div class="text-2xl font-bold text-green-900">{formatNumber(pool.funds)} {pool.token.symbol}</div>
      </div>
      
      <div class="bg-purple-50 p-4 rounded-lg">
        <div class="text-sm text-purple-800">Reward Per Demo</div>
        <div class="text-2xl font-bold text-purple-900">{pool.pricePerDemo || 1} {pool.token.symbol}</div>
      </div>
    </div>
    
    <!-- Summary Data -->
    <div class="mb-4">
      <h4 class="text-sm font-bold text-gray-700 mb-2">Deposit Address</h4>
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
  </Card>
  
  <!-- Status Section -->
  {#if pool.status === TrainingPoolStatus.noGas || pool.status === TrainingPoolStatus.noFunds}
    <div class="p-4 my-3 bg-amber-50 border-l-4 border-amber-400 rounded-lg flex gap-3 shadow-sm">
      <div class="text-amber-500 flex-shrink-0">
        <AlertTriangle size={18} />
      </div>
      <div class="flex-1">
        {#if pool.status === TrainingPoolStatus.noGas}
          <p class="text-sm text-amber-900 font-semibold mb-1">
            Insufficient SOL for Gas
          </p>
          <p class="text-xs leading-relaxed text-amber-800">
            Your gym needs SOL to pay for on-chain transactions. Without gas, the gym cannot function on the Solana blockchain.
          </p>
        {:else if pool.status === TrainingPoolStatus.noFunds}
          <p class="text-sm text-amber-900 font-semibold mb-1">
            Insufficient VIRAL Tokens
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
</div>
