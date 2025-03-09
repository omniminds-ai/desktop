<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { History, Settings, LayoutDashboard, Network, HomeIcon, House, Home, Zap, Shield } from 'lucide-svelte';
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
      case '/app/gym/privacy':
        return 'Privacy Controls';
      default:
        return 'Recording Details';
    }
  });
</script>

<div class=" max-w-7xl mx-auto mb-6">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-title text-secondary-300">{title}</h2>
    <div class="flex gap-3">
      <a 
        href="/app/gym" 
        class="text-secondary-300 hover:scale-115 transform transition-all  p-1 {page.route.id === '/app/gym' ? 'bg-secondary-100/20 rounded-md' : ''}">
        <Home class="w-6 h-6" />
      </a>
      <a 
        href="/app/gym/skills" 
        class="text-secondary-300 hover:scale-115 transform transition-all  p-1 {page.route.id === '/app/gym/skills' ? 'bg-secondary-100/20 rounded-md' : ''}">
        <Zap class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/history"
        class="text-secondary-300 hover:scale-115 transform transition-all  p-1 {page.route.id?.includes('/app/gym/history') ? 'bg-secondary-100/20 rounded-md' : ''}">
        <History class="w-6 h-6" />
      </a>
      <a
        href="/app/gym/privacy"
        class="text-secondary-300 hover:scale-115 transform transition-all  p-1 {page.route.id === '/app/gym/privacy' ? 'bg-secondary-100/20 rounded-md' : ''}">
        <Shield class="w-6 h-6" />
      </a>
    </div>
  </div>
</div>
