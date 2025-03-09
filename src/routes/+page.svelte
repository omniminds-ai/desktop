<script lang="ts">
  import { goto } from '$app/navigation';
  import { getPlatform } from '$lib/utils';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      if ((await getPlatform()) === 'macos') {
        // Check if accessibility permissions are already granted
        const hasPerms = (await invoke('has_ax_perms')) && (await invoke('has_record_perms'));
        const onboardingComplete = await invoke('get_onboarding_complete');

        if (!onboardingComplete) {
          goto('/onboarding');
          return;
        }

        // If permissions are not granted but we've already onboarded, show the ax page
        if (!hasPerms && onboardingComplete) {
          goto('/onboarding/ax');
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
