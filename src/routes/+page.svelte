<script lang="ts">
  import { goto } from '$app/navigation';
  import { getPlatform } from '$lib/utils';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      if ((await getPlatform()) === 'macos') {
        // Check if accessibility permissions are already granted
        const hasPermission =
          (await invoke('has_ax_perms')) &&
          // (await invoke('has_input_perms')) &&
          (await invoke('has_record_perms'));

        // If permissions are not granted, show the onboarding page
        if (!hasPermission) {
          goto('/onboarding');
          return;
        }
      }

      // Otherwise, go to the main app
      goto('/app/gym');
    } catch (error) {
      console.error('Error during startup:', error);
      // If there's an error, default to the main app
      goto('/app/gym');
    }
  });
</script>
