<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import {
    History,
    Settings,
    LayoutDashboard,
    Network,
    HomeIcon,
    House,
    Home,
    Zap
  } from 'lucide-svelte';

  const { hasPendingRewards = false } = $props();
  let title = $derived.by(() => {
    let path = page.route.id;
    // handle dynamic history routes
    if (path?.includes('history/')) path = '/app/gym/history/id';
    switch (path) {
      case '/app/gym':
        return 'Training Gym';
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

<div class="   mx-auto mb-6">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-title text-secondary-300">{title}</h2>
    <div class="flex gap-3">
      <a
        href="/app/gym"
        class="text-secondary-300 hover:scale-115 transform transition-all p-1 {page.route.id ===
        '/app/gym'
          ? 'bg-secondary-100/20 rounded-md'
          : ''}">
        <Home class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/skills"
        class="text-secondary-300 hover:scale-115 transform transition-all p-1 {page.route.id ===
        '/app/gym/skills'
          ? 'bg-secondary-100/20 rounded-md'
          : ''}">
        <Zap class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/history"
        class="text-secondary-300 hover:scale-115 transform transition-all p-1 {page.route.id?.includes(
          '/app/gym/history'
        )
          ? 'bg-secondary-100/20 rounded-md'
          : ''} relative">
        <History class="w-6 h-6" />
        {#if hasPendingRewards}
          <span class="absolute bottom-5.5 left-5.5 1 w-2.5 h-2.5 bg-secondary-100/50 rounded-full">
          </span>
        {/if}
      </a>
      <!-- <a
        href="/app/gym/settings"
        class="text-secondary-300 hover:scale-115 transform transition-all  p-1 {page.route.id === '/app/gym/settings' ? 'bg-secondary-100/20 rounded-md' : ''}">
        <Settings class="w-6 h-6" />
      </a> -->
    </div>
  </div>
</div>
