<script lang="ts">
import { MessageSquare, Dumbbell, Server, TestTube2 } from 'lucide-svelte';
import { page } from '$app/state';

import logo from '$lib/assets/Logo_Standard_white.png';

const buttons = [
  { path: '/', icon: MessageSquare, label: 'Chat' },
  { path: '/gym', icon: Dumbbell, label: 'Gym' },
  { path: '/node', icon: Server, label: 'Node' },
  { path: '/lab', icon: TestTube2, label: 'Lab' }
];
</script>

<div class="w-48 bg-transparent p-4 space-y-2">
  <div class="mb-4 flex items-center">
    <img src={logo} alt="ViralMind Logo" class="h-8 object-contain" />
  </div>
  
  {#each buttons as button}
    {@const Icon = button.icon}
    <a
      href={button.path}
      class="w-full py-2 px-4 flex items-center gap-3 rounded-full transition-colors {
        page.url.pathname === button.path 
          ? 'bg-[var(--vm-secondary-300)] text-white' 
          : 'hover:bg-white/10 text-gray-300'
      }"
    >
      <Icon size={20} />
      <span class="font-['Poppins'] font-bold">{button.label}</span>
    </a>
  {/each}
</div>
