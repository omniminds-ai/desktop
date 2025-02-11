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
  <div class="w-full py-2 px-4 flex items-center gap-2 rounded-lg bg-white/5">
    <Wallet size={16} class="text-gray-400" />
    <div class="flex-1 truncate text-sm text-gray-300">
      {$walletAddress.slice(0, 4)}...{$walletAddress.slice(-4)}
    </div>
    <div class="group relative flex">
      <button
        class="text-gray-400 relative cursor-pointer hover:text-white transition-colors"
        on:click={disconnectWallet}>
        <LogOut class="group" size={16} />
      </button>
      <span
        class="invisible text-center border border-gray-300/40 group-hover:visible opacity-0 group-hover:opacity-100 transition-all duration-200 absolute -top-13 -left-12 -translate-x-1/2 w-36 p-2 bg-black/10 text-gray-300 text-sm rounded-lg whitespace-nowrap">
        Disconnect Wallet
      </span>
    </div>
  </div>
{:else}
  <a
    href={getConnectionUrl()}
    target="_blank"
    class="w-full py-2 px-4 flex items-center justify-center gap-2 rounded-lg bg-black text-white hover:bg-neutral-800 transition-colors"
    on:click={() => startPolling()}>
    <Wallet size={16} />
    <span class="font-['Poppins'] font-bold">
      {$isConnecting ? 'Connecting...' : 'Connect'}
    </span>
  </a>
{/if}
