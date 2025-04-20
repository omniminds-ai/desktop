<script lang="ts">
  import { LoaderCircle, Send } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo_Icon.png';
  import { invoke } from '@tauri-apps/api/core';

  let request: string = $state('');
  let loading = $state(false);

  const executeRequest = async () => {
    loading = true;
    await invoke('run_demo_request', { request });
    loading = false;
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
    <div class="flex w-full bg-white rounded-xl p-2 gap-2">
      <input
        type="text"
        bind:value={request}
        disabled={loading}
        onkeydown={inputKeypress}
        class="w-full text-lg focus:outline-none"
        placeholder="Ask VM-0.1 to do anything." />
      <button
        onclick={executeRequest}
        class="text-white bg-secondary-300 p-2 rounded-full hover:bg-secondary-100 transition shadow-md hover:shadow-lg"
        title="Go">
        {#if !loading}
          <Send size={16} />
        {:else}
          <LoaderCircle class="animate-spin" size={16} />
        {/if}
      </button>
    </div>
  </div>
</div>
