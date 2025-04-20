<script lang="ts">
  import { LoaderCircle, Send, Circle } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo_Icon.png';
  import { invoke } from '@tauri-apps/api/core';
  import { fly, fade } from 'svelte/transition';

  let request: string = $state('');
  let loading = $state(false);
  let showInput = $state(true);
  let showDots = $state(false);
  let dotPattern = $state(0);

  // Cycle through dot patterns
  const cycleDots = () => {
    const patterns = 4; // Number of different patterns
    dotPattern = (dotPattern + 1) % patterns;
  };

  let dotInterval: number;

  const executeRequest = async () => {
    // Hide input and show dots
    showInput = false;
    loading = true;

    // Wait for animation to complete before showing dots
    setTimeout(() => {
      showDots = true;
      dotInterval = setInterval(cycleDots, 500);
      invoke('run_demo_request', { request }).finally(() => {
        //todo: remove this timeout
        setTimeout(() => {
          clearInterval(dotInterval);
          loading = false;
          showDots = false;
          showInput = true;
        }, 5000);
      });
    }, 500); // Match this with the duration of the fly transition
  };

  const inputKeypress = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      executeRequest();
    }
  };
</script>

<div class="w-full bg-transparent overflow-hidden max-h-[70px] h-[70px]">
  <div
    class="flex items-center align-middle rounded-2xl gap-2 px-3 py-2 border-white border-2 bg-primary-400 backdrop-blur-2xl text-gray-700">
    <div>
      <img src={Logo} alt="Logo" class="mx-auto w-12 h-8" />
    </div>
    <div class="flex w-full bg-white rounded-xl p-2 gap-2 relative">
      <div class="flex w-full gap-2">
        {#if showInput}
          <input
            in:fade={{ duration: 100 }}
            out:fly={{ x: 1000, duration: 500 }}
            type="text"
            bind:value={request}
            disabled={loading}
            onkeydown={inputKeypress}
            class="w-full text-lg focus:outline-none"
            placeholder="Ask VM-0.1 to do anything." />
        {/if}
        {#if showDots}
          <div class="flex items-start mt-3 w-full h-full" in:fade={{ duration: 300 }}>
            <div class="flex space-x-2">
              <!-- Pattern 0: . . . -->
              {#if dotPattern === 0}
                <Circle size={8} fill="#6366f1" stroke="none" />
                <Circle size={8} fill="#6366f1" stroke="none" />
                <Circle size={8} fill="#6366f1" stroke="none" />
                <!-- Pattern 1: . -->
              {:else if dotPattern === 1}
                <Circle size={8} fill="#6366f1" stroke="none" />
                <div class="w-2 h-2"></div>
                <div class="w-2 h-2"></div>
                <!-- Pattern 2: . . -->
              {:else if dotPattern === 2}
                <Circle size={8} fill="#6366f1" stroke="none" />
                <Circle size={8} fill="#6366f1" stroke="none" />
                <div class="w-2 h-2"></div>
                <!-- Pattern 3: . . . -->
              {:else}
                <Circle size={8} fill="#6366f1" stroke="none" />
                <Circle size={8} fill="#6366f1" stroke="none" />
                <Circle size={8} fill="#6366f1" stroke="none" />
              {/if}
            </div>
          </div>
        {/if}
        <button
          onclick={executeRequest}
          class="text-white bg-secondary-300 p-2 rounded-full hover:bg-secondary-100 transition shadow-md hover:shadow-lg"
          title="Go">
          {#if !loading}
            <Send size={16} />
          {:else}
            <LoaderCircle size={16} class="animate-spin" />
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>
