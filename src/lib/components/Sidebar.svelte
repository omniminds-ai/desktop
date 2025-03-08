<script lang="ts">
  import { MessageSquare, Dumbbell, Hammer } from 'lucide-svelte';
  import { page } from '$app/state';
  import logo from '$lib/assets/Logo_Icon.png';
  import WalletButton from './WalletButton.svelte';
  import UploadManager from './UploadManager.svelte';

  const earnButtons = [
    { path: '/app/gym', icon: Dumbbell, label: 'Gym' }
    // { path: "/app/node", icon: Server, label: "Node" },
  ];

  const spendButtons = [
    { path: '/app/forge', icon: Hammer, label: 'Forge' }
    // { path: "/app/lab", icon: TestTube2, label: "Lab" },
  ];
</script>

<div class="w-16 flex flex-col bg-transparent p-2 space-y-4">
  <div class="flex-start grow">
    <div class="my-4 flex justify-center">
      <img src={logo} alt="ViralMind Logo" class="h-8 w-8 object-contain" />
    </div>
    <a
      href="/app/chat"
      class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname ===
      '/app/chat'
        ? 'bg-secondary-300 text-white'
        : 'hover:bg-white/10 text-gray-300'}"
      title="Chat">
      <MessageSquare size={20} />
    </a>

    <div class="py-2">
      <div class="text-gray-400 py-1 text-xs text-center font-semibold">Earn</div>
      {#each earnButtons as button}
        {@const Icon = button.icon}
        <a
          href={button.path}
          class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname.startsWith(
            button.path
          )
            ? 'bg-secondary-300 text-white'
            : 'hover:bg-white/10 text-gray-300'}"
          title={button.label}>
          <Icon size={20} />
        </a>
      {/each}
    </div>

    <div class="py-2">
      <div class="text-gray-400 py-1 text-xs text-center font-semibold">Spend</div>
      {#each spendButtons as button}
        {@const Icon = button.icon}
        <a
          href={button.path}
          class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname.startsWith(
            button.path
          )
            ? 'bg-secondary-300 text-white'
            : 'hover:bg-white/10 text-gray-300'}"
          title={button.label}>
          <Icon size={20} />
        </a>
      {/each}
    </div>
  </div>
  <div class="self-end w-full">
    <UploadManager />
    <br />
    <WalletButton />
  </div>
</div>
