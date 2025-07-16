<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Button from '$lib/components/form/Button.svelte';
  import { Video, Check, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import AppText from './AppText.svelte';
  import { RecordingState, type Quest, type Screen } from '$lib/types/gym';
  import { recordingState } from '$lib/stores/recording';
  import ScreenSelectModal from './ScreenSelectModal.svelte';

  export let title: string;
  export let objectives: string[];
  export let reward: Quest['reward'];
  export let onStartRecording: (selectedScreen: Screen) => void;
  export let onComplete: () => void;
  export let onGiveUp: () => void;

  let recordingLoading = false;
  let ready = false;
  let screenshot = '';
  const scale = tweened(1, {
    duration: 60000,
    easing: cubicOut
  });

  let showScreenModal = false;
  let availableScreens: Screen[] = [];

  recordingState.subscribe((state) => {
    if (state === RecordingState.starting || state == RecordingState.saving) {
      recordingLoading = true;
    } else {
      recordingLoading = false;
    }
  });

  async function handleStartRecording() {
    if(availableScreens.length > 1) {
      showScreenModal = true;
      // setTimeout(async () => {
      //   await getAllScreens()
      // }, 500)
    } else {
      await handleScreenSelect(availableScreens[0].id)
    }
  }

  async function handleScreenSelect(id: string) {
    const selectedScreen = availableScreens.find(s => s.id === id);
    showScreenModal = false;
    if(selectedScreen) {
      onStartRecording(selectedScreen);
    }
  }

  async function getAllScreens() {
    const all = await invoke('capture_all_monitors');
    availableScreens = Array.isArray(all) ? all : [];
  }
  async function init() {
    try {
      setTimeout(async () => {
        screenshot = await invoke('take_screenshot');
        await getAllScreens()
        ready = true;
      }, 0)
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
    <div class="absolute top-5 right-6">
      {#if reward}
        <div class="flex flex-col self-end items-end">
          <div class="text-[10px] uppercase tracking-wider text-emerald-400/80 font-medium">
            Up to
          </div>
          <div
            class="flex flex-row items-center gap-1 px-2 py-1 bg-emerald-600/20 text-emerald-400 rounded-lg">
            <span class="text-sm font-bold">{reward.max_reward} {reward.symbol}</span>
          </div>
        </div>
      {/if}
    </div>
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
    {#if $recordingState === RecordingState.recording}
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
      <Button
        disabled="{!ready}"
        onclick={handleStartRecording}
        class="w-full flex! group items-center justify-center gap-2">
        {#if recordingLoading}
          <div
            class="w-6 h-6 rounded-full border-2 border-white group-hover:border-secondary-300 border-t-transparent! animate-spin">
          </div>
          Start Recording
        {:else}
          <Video size={20} />
          Start Recording
        {/if}
      </Button>
    {/if}
  </div>
</div>

{#if showScreenModal}
  <ScreenSelectModal
    screens={availableScreens}
    onSelect={handleScreenSelect}
    onClose={() => (showScreenModal = false)}
  />
{/if}
