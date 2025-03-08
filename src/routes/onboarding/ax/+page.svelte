<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { relaunch } from '@tauri-apps/plugin-process';
  import Button from '$lib/components/Button.svelte';
  import { getPlatform } from '$lib/utils';
  import { goto } from '$app/navigation';

  // State for tracking permissions
  let isMacOS = true;
  let accessibilityGranted = $state(false);
  let screenRecordingGranted = $state(false);
  let loading = $state(true);

  // Check if we're on macOS
  onMount(async () => {
    // Check platform
    const platform = await getPlatform();
    loading = false;

    // If not on macOS, we don't need to show this page
    if (platform !== 'macos') {
      console.log('Not on macos, skipping permissions');
      await invoke('set_onboarding_complete', { confirmed: true });
      return goto('/app/gym');
    }

    accessibilityGranted = await invoke('has_ax_perms');
    screenRecordingGranted = await invoke('has_record_perms');

    if (accessibilityGranted && screenRecordingGranted) {
      await invoke('set_onboarding_complete', { confirmed: true });
      return goto('/app/gym');
    }
  });

  // Request accessibility permissions
  async function requestAccessibilityPermission() {
    try {
      // macOS will show the system dialog
      await invoke('request_ax_perms');
      accessibilityGranted = true;
    } catch (error) {
      console.error('Error requesting accessibility permissions:', error);
    }
  }

  // Request screen recording permissions
  async function requestScreenRecordingPermission() {
    try {
      await invoke('request_record_perms');
      screenRecordingGranted = true;
    } catch (error) {
      console.error('Error requesting screen recording permissions:', error);
    }
  }

  // Restart the app
  async function restartApp() {
    try {
      await invoke('set_onboarding_complete');
      await relaunch();
    } catch (error) {
      console.error('Error restarting app:', error);
    }
  }

  // Check if all permissions are granted
  const allPermissionsGranted = $derived(accessibilityGranted && screenRecordingGranted);
</script>

<div class="max-w-3xl w-full bg-white rounded-2xl p-8 shadow-md text-black">
  <h1 class="text-4xl mb-4 text-secondary-300 text-center font-title">Almost Done!</h1>

  {#if loading}
    <p class="text-center">Loading...</p>
  {:else}
    <p class="text-xl mb-8 text-center">
      To use all features of Viralmind Desktop, we need the following permissions:
    </p>

    <div class="flex flex-col gap-6 mb-8">
      <!-- Accessibility Permission -->
      <div
        class={`bg-gray-100 shadow-md rounded-lg p-6 border-l-4 ${accessibilityGranted ? 'border-emerald-500 bg-opacity-10 bg-emerald-500' : 'border-red-600'} transition-all duration-300`}>
        <div class="flex justify-between items-center mb-2">
          <h3 class="text-xl font-title m-0">1. Accessibility Permission</h3>
          {#if accessibilityGranted}
            <span class="text-sm font-bold py-1 px-2 rounded bg-emerald-500 text-white">
              ✓ Pending
            </span>
          {:else}
            <span class="text-sm font-bold py-1 px-2 rounded bg-red-600 text-white">Denied</span>
          {/if}
        </div>
        <p class="mb-4 leading-relaxed">
          This allows Viralmind to understand what's on your screen and provide relevant assistance.
        </p>
        {#if !accessibilityGranted}
          <Button variant="primary" onclick={requestAccessibilityPermission}>
            Grant Accessibility Permission
          </Button>
        {/if}
      </div>
      <!-- Screen Recording Permission -->
      <div
        class={`bg-gray-100 shadow-md rounded-lg p-6 border-l-4 ${screenRecordingGranted ? 'border-emerald-500 bg-opacity-10 bg-emerald-500' : 'border-red-600'} transition-all duration-300`}>
        <div class="flex justify-between items-center mb-2">
          <h3 class="text-xl font-title m-0">2. Screen Recording Permission</h3>
          {#if screenRecordingGranted}
            <span class="text-sm font-bold py-1 px-2 rounded bg-emerald-500 text-white">
              ✓ Pending
            </span>
          {:else}
            <span class="text-sm font-bold py-1 px-2 rounded bg-red-600 text-white">Denied</span>
          {/if}
        </div>
        <p class="mb-4 leading-relaxed">
          This allows Viralmind to record your screen for training and assistance purposes.
        </p>
        {#if !screenRecordingGranted}
          <Button variant="primary" onclick={requestScreenRecordingPermission}>
            Grant Screen Recording Permission
          </Button>
        {/if}
      </div>
    </div>

    <div class="flex justify-center mt-8">
      <Button variant="green" disabled={!allPermissionsGranted} onclick={restartApp}>
        {allPermissionsGranted
          ? 'Restart for Permissions to Take Effect'
          : 'Please Grant All Permissions'}
      </Button>
    </div>
  {/if}
</div>
