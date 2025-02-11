<script lang="ts">
  import { History, Settings, LayoutDashboard } from 'lucide-svelte';
  export let title: string;
</script>

<div class="flex justify-between items-center mb-6">
  <h2 class="text-2xl font-title text-secondary-300">{title}</h2>
  <div class="flex gap-4">
    <a href="/app/gym" class="text-secondary-300 hover:text-secondary-400 transition-colors">
      <LayoutDashboard class="w-6 h-6" />
    </a>
    <a href="/app/gym/history" class="text-secondary-300 hover:text-secondary-400 transition-colors">
      <History class="w-6 h-6" />
    </a>
    <a href="/app/gym/settings" class="text-secondary-300 hover:text-secondary-400 transition-colors">
      <Settings class="w-6 h-6" />
    </a>
  </div>
</div>
