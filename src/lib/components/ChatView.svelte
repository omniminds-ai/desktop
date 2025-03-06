<script lang="ts">
  import xLogo from '$lib/assets/x_logo.svg';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { Send, Bot, User, Keyboard, Mouse, Gamepad, Monitor, Twitter, X } from 'lucide-svelte';
  import { onMount, onDestroy } from 'svelte';
  import Card from './Card.svelte';
  import Button from './Button.svelte';
  import Input from './Input.svelte';

  let name = $state('');
  let greetMsg = $state('');
  let inputEvents = $state<any[]>([]);
  const MAX_EVENTS = 10;

  async function greet(event: Event) {
    event.preventDefault();
    greetMsg = await invoke('greet', { name });
  }

  // Listen for global input events from Rust
  listen<any>('input-event', (event: { payload: any }) => {
    inputEvents = [event.payload, ...inputEvents.slice(0, MAX_EVENTS - 1)];
  });

  function formatEvent(event: any): string {
    const { type, data } = event;
    switch (type) {
      case 'KeyPress':
        return `Key pressed: ${data.key}`;
      case 'KeyRelease':
        return `Key released: ${data.key}`;
      case 'MouseMove':
        return `Mouse position: (${data.x.toFixed(1)}, ${data.y.toFixed(1)})`;
      case 'MouseDelta':
        return `Mouse delta: (${data.x}, ${data.y})`;
      case 'MouseClick':
        return `Mouse ${data.state.toLowerCase()}: ${data.button}`;
      case 'MouseWheel':
        return `Mouse wheel: ${data.delta}`;
      case 'JoystickButton':
        return `Joystick ${data.id} button ${data.button}: ${data.state.toLowerCase()}`;
      case 'JoystickAxis':
        return `Joystick ${data.id} axis ${data.axis}: ${data.value.toFixed(2)}`;
      default:
        return JSON.stringify(event);
    }
  }
</script>

<div class="relative h-full flex flex-col">
  <!-- TEMPORARY DISABLED COVER -->
  <!-- <div class="absolute h-full w-full z-50 flex items-center align-middle">
    <div class="mx-auto text-center w-full">
      <p class="font-semibold text-lg">Viralmind chat is coming soon!</p>
      <p class="text-slate-600">Stay tuned by following us on X.</p>
      <a
        target="_blank"
        href="https://x.com/ViralMindAI"
        class="text-center text-secondary-300 w-full hover:underline rounded-lg mt-2">
        <span>@viralmindai</span>
      </a>
    </div>
  </div> -->

  <div class="h-full flex flex-col">
    <div class="flex-1 p-6 space-y-6 overflow-y-auto">
      <div class="flex gap-4">
        <div
          class="shrink-0 w-8 h-8 rounded-full bg-secondary-300 text-white flex items-center justify-center shadow-sm">
          <Bot size={18} />
        </div>
        <Card variant="secondary" className="max-w-2xl shadow-sm">
          Hello! I'm V, your ViralMind assistant. What desktop task would you like me to perform?
        </Card>
      </div>
    </div>

    {#if inputEvents.length > 0}
      <div class="flex gap-4">
        <div
          class="shrink-0 w-8 h-8 rounded-full bg-secondary-300 text-white flex items-center justify-center shadow-sm">
          {#if inputEvents.length > 0 && inputEvents[0].type.startsWith('Key')}
            <Keyboard size={18} />
          {:else if inputEvents[0].type.startsWith('Joystick')}
            <Gamepad size={18} />
          {:else}
            <Mouse size={18} />
          {/if}
        </div>
        <Card variant="secondary" className="max-w-2xl shadow-sm">
          <div class="space-y-2">
            {#each inputEvents as event}
              <div class="text-sm font-mono">{formatEvent(event)}</div>
            {/each}
          </div>
        </Card>
      </div>
    {/if}
  </div>
  
  <div>
    <div class="p-4 bg-white border-t border-gray-200">
      <div class="flex gap-3 items-center max-w-4xl mx-auto">
        <form class="flex-1 flex gap-3 items-center" onsubmit={greet}>
          <Input
            type="text"
            variant="light"
            placeholder="Type your message..."
            bind:value={name}
            class="flex-1 px-6 py-3 rounded-full! transition-colors" />
          <Button type="submit" class="rounded-full">
            <Send size={20} />
          </Button>
        </form>
      </div>
    </div>
  </div>
</div>
