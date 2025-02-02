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
  <div class="flex-1 p-6 space-y-6 overflow-y-auto">
    <div class="flex gap-4">
      <div class="shrink-0 w-8 h-8 rounded-full bg-[var(--vm-secondary-300)] text-white flex items-center justify-center shadow-sm">
        <Bot size={18} />
      </div>
      <Card variant="secondary" className="max-w-2xl shadow-sm">
        Hello! I'm your ViralMind assistant. What desktop task would you like me to perform?
      </Card>
    </div>
    
    {#if greetMsg}
      <div class="flex gap-4 justify-end">
        <Card variant="primary" className="max-w-2xl shadow-sm">
          {greetMsg}
        </Card>
        <div class="shrink-0 w-8 h-8 rounded-full bg-[var(--vm-secondary-100)] text-white flex items-center justify-center shadow-sm">
          <User size={18} />
        </div>
      </div>
    {/if}
  </div>

  <div class="p-4 bg-white border-t border-gray-200">
    <form class="flex gap-3 items-center max-w-4xl mx-auto" onsubmit={greet}>
      <input 
        type="text" 
        placeholder="Type your message..."
        bind:value={name}
        class="flex-1 px-6 py-3 rounded-full border border-gray-200 bg-gray-50 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)] focus:bg-white transition-colors"
      />
      <button 
        type="submit" 
        class="p-3 rounded-full bg-[var(--vm-secondary-300)] text-white hover:bg-[var(--vm-secondary-400)] transition-colors shadow-sm hover:shadow-md"
      >
        <Send size={20} />
      </button>
    </form>
  </div>
</div>
