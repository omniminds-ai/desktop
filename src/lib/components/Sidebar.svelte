<script lang="ts">
  import { Dumbbell, Hammer, Square, LoaderCircle, Trophy } from 'lucide-svelte';
  import { page } from '$app/state';
  import logo from '$lib/assets/Logo_Icon.png';
  import WalletButton from './WalletButton.svelte';
  import UploadManager from './UploadManager.svelte';
  import { stopRecording } from '$lib/api/endpoints/gym';
  import { RecordingState } from '$lib/types/gym';
  import { recordingState } from '$lib/stores/recording';

  const earnButtons = [
    { path: '/app/gym', icon: Dumbbell, label: 'Gym' },
    { path: '/app/leaderboards', icon: Trophy, label: 'Loaderboards' }
  ];

  const spendButtons = [
    { path: '/app/forge', icon: Hammer, label: 'Forge' }
    // { path: "/app/lab", icon: TestTube2, label: "Lab" },
  ];

  // Recording state
  let recordingLoading = false;

  recordingState.subscribe((state) => {
    if (state === RecordingState.starting || state == RecordingState.saving) {
      recordingLoading = true;
    } else {
      recordingLoading = false;
    }
  });

  // Handle stop recording click
  async function handleStopRecording() {
    try {
      await stopRecording();
    } catch (error) {
      console.error('Failed to stop recording:', error);
    }
  }
</script>

<div class="w-16 flex flex-col bg-transparent py-2 pl-2 space-y-4">
  <div class="flex-start grow flex flex-col gap-2">
    <div class="my-4 flex justify-center">
      <img src={logo} alt="ViralMind Logo" class="h-8 w-8 object-contain" />
    </div>

    <!-- <a
      href="/app/chat"
      class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname ===
      '/app/chat'
        ? 'bg-secondary-300 text-white'
        : 'hover:bg-white/10 text-gray-300'}"
      title="Chat">
      <MessageSquare size={20} />
    </a> -->

    <!-- <a
      href="/app/datasets"
      class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname.split('/')[2] == 'datasets'
        ? 'bg-secondary-300 text-white'
        : 'hover:bg-white/10 text-gray-300'}"
      title="Datasets">
      <Database size={20} />
    </a> -->

    <div class="py-2">
      <div class="text-gray-400 py-1 text-xs text-center font-semibold">Earn</div>
      {#each earnButtons as button}
        {@const Icon = button.icon}
        <a
          href={button.path}
          class="w-full my-1 py-2 flex justify-center rounded-full transition-colors {page.url.pathname.split(
            '/'
          )[2] == button.path.split('/')[2]
            ? 'bg-secondary-300 text-white'
            : 'hover:bg-white/10 text-gray-300'}"
          title={button.label}>
          <Icon size={20} />
        </a>
      {/each}
    </div>

    <div class="py-2">
      <div class="text-gray-400 py-1 text-xs text-center font-semibold">Forge</div>
      {#each spendButtons as button}
        {@const Icon = button.icon}
        <a
          href={button.path}
          class="w-full py-2 flex justify-center rounded-full transition-colors {page.url.pathname.split(
            '/'
          )[2] == button.path.split('/')[2]
            ? 'bg-secondary-300 text-white'
            : 'hover:bg-white/10 text-gray-300'}"
          title={button.label}>
          <Icon size={20} />
        </a>
      {/each}
    </div>
  </div>
  <div class="self-end w-full">
    <!-- Recording Indicator -->
    {#if $recordingState == RecordingState.recording}
      <button
        class="rounded-full mx-auto w-12 h-12 p-3 bg-red-500 text-white hover:bg-red-600 transition-colors full flex justify-center items-center mb-2"
        on:click={handleStopRecording}
        title="Stop Recording">
        <Square class="w-full h-full" />
      </button>
    {:else if recordingLoading}
      <button
        disabled
        class="rounded-full mx-auto w-12 h-12 p-3 bg-red-500 text-white transition-colors full flex justify-center items-center mb-2"
        title="Stop Recording">
        <LoaderCircle class="w-full animate-spin h-full" />
      </button>
    {/if}
    <UploadManager />
    <br />
    <WalletButton />
  </div>
</div>
