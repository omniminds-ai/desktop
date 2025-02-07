<script lang="ts">
  import { MessageSquare, Dumbbell, Server, TestTube2, Workflow, Hammer, Wallet, LogOut } from "lucide-svelte";
  import { page } from "$app/state";
  import { walletAddress, isConnecting, getConnectionUrl, startPolling, disconnectWallet } from "$lib/stores/wallet";
  import logo from "$lib/assets/Logo_Standard_white.png";

  const earnButtons = [
    { path: "/app/gym", icon: Dumbbell, label: "Gym" },
    { path: "/app/node", icon: Server, label: "Node" },
  ];

  const spendButtons = [
    { path: "/app/forge", icon: Hammer, label: "Forge" },
    { path: "/app/lab", icon: TestTube2, label: "Lab" },
  ];
</script>

<div class="w-48 bg-transparent p-4 space-y-4">
  <div class="mb-4 flex items-center">
    <img src={logo} alt="ViralMind Logo" class="h-8 object-contain" />
  </div>

  <div class="absolute bottom-0 left-0 w-48 p-4">
    {#if $walletAddress}
      <div class="w-full py-2 px-4 flex items-center gap-2 rounded-lg bg-white/5">
        <Wallet size={16} class="text-gray-400" />
        <div class="flex-1 truncate text-sm text-gray-300">
          {$walletAddress.slice(0, 4)}...{$walletAddress.slice(-4)}
        </div>
        <button 
          class="text-gray-400 hover:text-white transition-colors"
          on:click={disconnectWallet}
        >
          <LogOut size={16} />
        </button>
      </div>
    {:else}
      <a
        href={getConnectionUrl()}
        target="_blank"
        class="w-full py-2 px-4 flex items-center justify-center gap-2 rounded-lg bg-black text-white hover:bg-neutral-800 transition-colors"
        on:click={() => startPolling()}
      >
        <Wallet size={16} />
        <span class="font-['Poppins'] font-bold">
          {$isConnecting ? 'Connecting...' : 'Connect'}
        </span>
      </a>
    {/if}
  </div>

  <a
    href="/app"
    class="w-full py-2 px-4 flex items-center gap-3 rounded-full transition-colors {page.url.pathname === '/app'
      ? 'bg-secondary-300 text-white'
      : 'hover:bg-white/10 text-gray-300'}"
  >
    <MessageSquare size={20} />
    <span class="font-['Poppins'] font-bold">Chat</span>
  </a>

  <div class="space-y-2">
    <div class="text-gray-400 text-sm font-semibold px-4">Earn</div>
    {#each earnButtons as button}
      {@const Icon = button.icon}
      <a
        href={button.path}
        class="w-full py-2 px-4 flex items-center gap-3 rounded-full transition-colors {page.url.pathname.startsWith(button.path)
          ? 'bg-secondary-300 text-white'
          : 'hover:bg-white/10 text-gray-300'}"
      >
        <Icon size={20} />
        <span class="font-['Poppins'] font-bold">{button.label}</span>
      </a>
    {/each}
  </div>

  <div class="space-y-2">
    <div class="text-gray-400 text-sm font-semibold px-4">Spend</div>
    {#each spendButtons as button}
      {@const Icon = button.icon}
      <a
        href={button.path}
        class="w-full py-2 px-4 flex items-center gap-3 rounded-full transition-colors {page.url.pathname.startsWith(button.path)
          ? 'bg-secondary-300 text-white'
          : 'hover:bg-white/10 text-gray-300'}"
      >
        <Icon size={20} />
        <span class="font-['Poppins'] font-bold">{button.label}</span>
      </a>
    {/each}
  </div>
</div>
