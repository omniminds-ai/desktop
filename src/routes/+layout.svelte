<script lang="ts">
  import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
  import '../app.css';
  import { onDestroy, onMount } from 'svelte';
  import { recordingState } from '$lib/stores/recording';
  import { RecordingState } from '$lib/types/gym';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  let { children } = $props();
  let unlistenState: UnlistenFn | null = null;

  onMount(async () => {
    // update recording state store
    unlistenState = await listen<{ state: RecordingState }>('recording-status', (event) => {
      $recordingState = event.payload.state;
    });
  });
  onDestroy(() => {
    unlistenState?.();
  });

  onOpenUrl(async (urls) => {
    console.log('Deep link opened!');
    console.log(urls);
  });
</script>

<svelte:head>Omniminds Desktop</svelte:head>

{@render children()}

<style>

</style>
