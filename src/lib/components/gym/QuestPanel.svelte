<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Button from '../Button.svelte';
  import { Video, Check, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import AppText from './AppText.svelte';

  export let title: string;
  export let objectives: string[];
  export let onStartRecording: () => void;
  export let onComplete: () => void;
  export let onGiveUp: () => void;
  export let recordingState: 'recording' | 'stopping' | 'stopped' = 'stopped';

  let screenshot = '';
  const scale = tweened(1, {
    duration: 60000,
    easing: cubicOut
  });

  async function handleStartRecording() {
    onStartRecording();
  }

  async function init() {
    try {
      screenshot = await invoke('take_screenshot');
      scale.set(1.5); // Start scale animation after screenshot is loaded
    } catch (error) {
      console.error('Failed to take screenshot:', error);
    }
  }

  init();
</script>

<div class="relative mt-4 mb-6 mx-4 rounded-lg overflow-hidden bg-black">
  {#if screenshot}
    <div
      class="absolute inset-0 bg-cover bg-center"
      style="background-image: url({screenshot}); filter: blur(2px) brightness(0.25); transform: scale({$scale});"
      in:fade={{ duration: 1000 }}>
    </div>
  {/if}

  <div class="relative p-6 bg-black/30">
    <h3 class="text-xl font-bold text-white mb-2">{title}</h3>
    <div class="mb-4">
      <h4 class="text-lg font-semibold text-white/90 mb-2">Objectives:</h4>
      <ul class="list-disc pl-5 space-y-1">
        {#each objectives as objective}
          <li class="text-white/80">
            <AppText text={objective} />
          </li>
        {/each}
      </ul>
    </div>
    {#if recordingState === 'recording'}
      <div class="flex gap-4">
        <Button variant="green" class="flex-1" onclick={onComplete}>
          <div class="flex items-center justify-center gap-2">
            <Check size={20} />
            <span>Complete</span>
          </div>
        </Button>
        <Button variant="destroy" class="flex-1" onclick={onGiveUp}>
          <div class="flex items-center justify-center gap-2">
            <X size={20} />
            <span>Give Up</span>
          </div>
        </Button>
      </div>
    {:else}
      <Button onclick={handleStartRecording} class="w-full flex! items-center justify-center gap-2">
        <Video size={20} />
        Start Recording
      </Button>
    {/if}
  </div>
</div>
