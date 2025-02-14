<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { History, Settings, LayoutDashboard, Network, ArrowLeft } from 'lucide-svelte';
  let title = $derived.by(() => {
    let path = page.route.id;
    // handle dynamic history routes
    if (path?.includes('history/')) path = '/app/gym/history/id';
    switch (path) {
      case '/app/gym':
        return 'Train AI and Earn';
      case '/app/gym/chat':
        return 'Training Session';
      case '/app/gym/skills':
        return 'Skill Tree';
      case '/app/gym/history':
        return 'Recording History';
      case '/app/gym/history/id':
        return 'Recording Details';
      case '/app/gym/settings':
        return 'Settings';
      default:
        return 'Recording Details';
    }
  });
</script>

<div class=" max-w-7xl mx-auto mb-6">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-title text-secondary-300">{title}</h2>
    <div class="flex gap-4">
      <a href="/app/gym" class="text-secondary-300 hover:scale-115 transform transition-all">
        <LayoutDashboard class="w-6 h-6" />
      </a>
      <a href="/app/gym/skills" class="text-secondary-300 hover:scale-115 transform transition-all">
        <Network class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/history"
        class="text-secondary-300 hover:scale-115 transform transition-all">
        <History class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/settings"
        class="text-secondary-300 hover:scale-115 transform transition-all">
        <Settings class="w-6 h-6" />
      </a>
    </div>
  </div>

  {#if page.route.id !== '/app/gym'}
    <button
      onclick={() => goto('/app/gym/')}
      class="text-secondary-300 text-sm cursor-pointer mb-2 flex gap-2 items-center hover:underline">
      <ArrowLeft size={15} strokeWidth={3} /> Gym Home
    </button>
  {/if}
</div>
