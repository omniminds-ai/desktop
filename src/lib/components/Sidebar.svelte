<script lang="ts">
  import {
    Square,
    CrownIcon,
    PencilRuler,
    Rocket,
    ArrowLeftRight,
    Users,
    FileText,
    BrainCog
  } from 'lucide-svelte';
  import { page } from '$app/state';
  import logo from '$lib/assets/Logo_Icon.png';
  import WalletButton from './WalletButton.svelte';
  import UploadManager from './UploadManager.svelte';
  import { stopRecording } from '$lib/api/endpoints/gym';
  import { RecordingState } from '$lib/types/gym';
  import { recordingState } from '$lib/stores/recording';

  const earnButtons = [
    { path: '/app/gym', icon: CrownIcon, label: 'The Arena' },
    { path: '/app/creator-hub', icon: PencilRuler, label: 'Creator Hub' },
    { path: '/app/launch-bay', icon: Rocket, label: 'LaunchBay' },
    { path: '/app/exchange', icon: ArrowLeftRight, label: 'The Exchange' },
    { path: '/app/my-agents', icon: Users, label: 'My Agents ' },
    { path: '/app/omni-docs', icon: FileText, label: 'OmniDocs ' },
    { path: '/app/settings', icon: BrainCog, label: 'Settings ' }
  ];

  const buttons = [...earnButtons];
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

<div class="w-[300px] flex flex-col bg-transparent pt-2 pb-6 pl-2 space-y-4">
  <div class="flex-start grow flex flex-col gap-2">
    <div class="my-4 flex justify-center">
      <img src={logo} alt="Omniminds Logo" class="w-[350px] h-[50px] object-contain" />
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

    <div class="py-2 px-6">
      {#each buttons as button}
        {@const Icon = button.icon}
        <a
          href={button.path}
          class="w-full my-3 py-2 flex items-center rounded-full transition-colors gap-5 font-medium text-xl {page.url.pathname.split(
            '/'
          )[2] == button.path.split('/')[2]
            ? 'bg-secondary-300 text-white'
            : 'hover:bg-white/10 text-gray-300'}"
          title={button.label}>
          <Icon size={20} class="ms-10" />
          {button.label}
        </a>
        {#if button !== buttons[buttons.length - 1]}
          <div class="h-[1px] bg-[#FFFFFF] opacity-15"></div>
        {/if}
      {/each}
    </div>
  </div>
  <div class="self-end w-full">
    <!-- Recording Indicator -->
    {#if $recordingState == RecordingState.recording}
      <button
        class="rounded-full mx-auto w-12 h-12 p-3 bg-red-500 text-white hover:bg-red-600 transition-colors full flex justify-center items-center mb-2"
        on:click={handleStopRecording}>
        <Square class="w-full h-full" />
      </button>
    {/if}
    <UploadManager />
    <br />
    <WalletButton />
  </div>
</div>
