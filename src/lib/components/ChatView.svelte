<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Send, Bot, User } from 'lucide-svelte';
import Card from './Card.svelte';

let name = $state("");
let greetMsg = $state("");

async function greet(event: Event) {
  event.preventDefault();
  greetMsg = await invoke("greet", { name });
}
</script>

<div class="h-full flex flex-col">
  <div class="flex-1 p-4 space-y-4">
    <div class="flex gap-4">
      <!-- <div class="aspect-square w-8 h-8 rounded-full bg-[var(--vm-secondary-300)] flex items-center justify-center">
        <Bot size={20} />
      </div> -->
      <Card variant="secondary" className="max-w-2xl">
        Hello! I'm your ViralMind assistant. What desktop task would you like me to perform?
      </Card>
    </div>
    
    {#if greetMsg}
      <div class="flex gap-4 justify-end">
        <Card variant="primary" className="max-w-2xl">
          {greetMsg}
        </Card>
        <!-- <div class="w-8 h-8 rounded-full bg-gray-300 flex items-center justify-center">
          <User size={20} />
        </div> -->
      </div>
    {/if}
  </div>

  <div class="p-4 border-t border-gray-200">
    <form class="flex gap-2 items-center" onsubmit={greet}>
      <input 
        type="text" 
        placeholder="Enter your name..."
        bind:value={name}
        class="flex-1 p-2 rounded-lg border border-gray-200 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)]"
      />
      <button type="submit" class="p-2 rounded-lg bg-[var(--vm-secondary-300)] text-white">
        <Send size={20} />
      </button>
    </form>
  </div>
</div>
