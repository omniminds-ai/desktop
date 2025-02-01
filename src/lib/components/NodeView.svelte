<script lang="ts">
import Card from './Card.svelte';
let showVerbose = $state(false);

const systemRequirements = [
  { name: 'WSL2', status: 'Enabled', ok: true },
  { name: 'Hyper-V', status: 'Required', ok: false }
];

const transactions = [
  { amount: 2.5, description: 'Desktop VM leased to node0x742F' },
  { amount: 1.8, description: 'Desktop VM leased to node0x891A' },
  { amount: -0.3, description: 'Commission paid to treasury' }
];

const logs = [
  '[INFO] Starting virald v1.2.3...',
  '[DEBUG] Checking system requirements...',
  '[INFO] WSL2 detected and running',
  '[WARN] Hyper-V not enabled',
  '[INFO] Port 8080 successfully bound',
  '[DEBUG] Loading VM configuration...',
  '[SUCCESS] Desktop VM pool initialized'
];
</script>

<div class="h-full space-y-6 p-4">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl">Viral Node</h2>
    <div class="flex items-center gap-2">
      <span class="text-green-500 font-medium">Earned: 1,245 $VIRAL</span>
      <button class="px-4 py-2 bg-green-500 text-white rounded-lg">Claim</button>
    </div>
  </div>
  
  <Card padding="lg">
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-4">
        <div class="w-3 h-3 rounded-full bg-green-500"></div>
        <span class="font-medium">virald is running</span>
      </div>
      <button class="px-4 py-2 bg-red-500 text-white rounded-lg">Stop Daemon</button>
    </div>

    <h3 class="text-lg mb-4">System Requirements</h3>
    <div class="grid grid-cols-3 gap-4 mb-6">
      {#each systemRequirements as req}
        <Card padding="sm" variant="secondary" className="flex items-center gap-3">
          <div class={req.ok ? 'text-green-500' : 'text-red-500'}>
            {#if req.ok}
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {:else}
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
              </svg>
            {/if}
          </div>
          <div>
            <div class="font-title">{req.name}</div>
            <div class="text-sm text-gray-500">{req.status}</div>
          </div>
        </Card>
      {/each}
    </div>

    <div class="border-t border-gray-200 pt-6">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-lg">Logs</h3>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-500">Show verbose</span>
          <button 
            onclick={() => showVerbose = !showVerbose}
            class={`w-12 h-6 rounded-full transition-colors ${showVerbose ? 'bg-[var(--vm-secondary-300)]' : 'bg-gray-300'}`}
          >
            <div class={`w-5 h-5 rounded-full bg-white transform transition-transform ${showVerbose ? 'translate-x-6' : 'translate-x-1'}`}></div>
          </button>
        </div>
      </div>
      
      {#if showVerbose}
        <div class="font-mono text-sm bg-gray-900 text-gray-100 p-4 rounded-lg space-y-1">
          {#each logs as log}
            <div class={
              log.includes('[SUCCESS]') ? 'text-green-400' :
              log.includes('[WARN]') ? 'text-yellow-400' :
              log.includes('[ERROR]') ? 'text-red-400' :
              ''
            }>{log}</div>
          {/each}
        </div>
      {:else}
        <div class="font-mono text-sm space-y-2">
          {#each transactions as tx}
            <Card variant={tx.amount > 0 ? 'secondary' : 'primary'} padding="sm" className="flex justify-between">
              <span>{tx.amount > 0 ? `+ ${tx.amount}` : '→ ' + Math.abs(tx.amount)} $VIRAL</span>
              <span>{tx.description}</span>
            </Card>
          {/each}
        </div>
      {/if}
    </div>
  </Card>
</div>
