<script lang="ts">
  import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
  import '../app.css';
  import ToastContainer from '$lib/components/ToastContainer.svelte';

  let { children } = $props();

  onOpenUrl(async (urls) => {
    console.log('Deep link opened!');
    console.log(urls);
  });
</script>

<svelte:head>viralmind desktop</svelte:head>

{@render children()}
