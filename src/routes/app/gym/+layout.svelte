<script lang="ts">
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  const { children } = $props();

  onMount(async () => {
    await invoke('request_record_perms');
    await invoke('request_input_perms');
  });
</script>

<div class="relative">
  <div class="p-2 sm:p-4">
    <GymHeader />
    {@render children()}
  </div>
</div>
