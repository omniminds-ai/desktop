<script lang="ts">
  import {
    walletAddress,
    isConnecting,
    getConnectionUrl,
    startPolling,
    disconnectWallet
  } from '$lib/stores/wallet';
  import { LogOut, Wallet } from 'lucide-svelte';
</script>

{#if $walletAddress}
  <div class="w-full py-2 flex justify-center rounded-lg bg-white/5 relative group">
    <Wallet size={16} class="text-gray-400" />
    <div class="invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-all duration-200 absolute -top-10 left-full ml-2 w-36 p-2 bg-black/80 text-gray-300 text-xs rounded-lg whitespace-nowrap text-center z-50">
      {$walletAddress.slice(0, 4)}...{$walletAddress.slice(-4)}
      <button
        class="ml-2 text-gray-400 cursor-pointer hover:text-white transition-colors"
        on:click={disconnectWallet}>
        <LogOut size={12} />
      </button>
    </div>
  </div>
{:else}
  <a
    href={getConnectionUrl()}
    target="_blank"
    class="w-full py-2 flex justify-center rounded-lg bg-black text-white hover:bg-neutral-800 transition-colors relative group"
    on:click={() => startPolling()}>
    <Wallet size={16} />
    <div class="invisible group-hover:visible opacity-0 group-hover:opacity-100 transition-all duration-200 absolute -top-8 left-full ml-2 w-24 p-1 bg-black/80 text-gray-300 text-xs rounded-lg whitespace-nowrap text-center z-50">
      {$isConnecting ? 'Connecting...' : 'Connect'}
    </div>
  </a>
{/if}
