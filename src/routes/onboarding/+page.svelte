<script lang="ts">
  import { goto } from '$app/navigation';
  import Logo from '$lib/assets/Logo_Icon.png';
  import Button from '$lib/components/form/Button.svelte';
  import WalletButton from '$lib/components/WalletButton.svelte';
  import { getPlatform } from '$lib/utils';
  import { privacyAccepted } from '$lib/stores/privacy';
  import { walletAddress } from '$lib/stores/wallet';
  import { invoke } from '@tauri-apps/api/core';
  import PrivacyPolicy from '$lib/components/PrivacyPolicy.svelte';

  // Reactive variable to check if both conditions are met
  const canContinue = $derived($privacyAccepted && $walletAddress !== null);

  const goNext = async () => {
    const platform = await getPlatform();

    const accessibilityGranted = await invoke('has_ax_perms');
    const screenRecordingGranted = await invoke('has_record_perms');

    if (platform === 'macos') {
      if (accessibilityGranted && screenRecordingGranted) {
        await invoke('set_onboarding_complete', { confirmed: true });
        goto('/app/gym');
      } else {
        goto('/onboarding/ax');
      }
    } else {
      await invoke('set_onboarding_complete', { confirmed: true });
      goto('/app/gym');
    }
  };
</script>

<div class="max-w-3xl w-full bg-white rounded-2xl px-8 py-4 shadow-lg text-black">
  <div class="flex-col flex items-center justify-center border-b border-gray-200 pb-6 mb-6">
    <img src={Logo} alt="Viralmind Logo" class="w-28 h-28 object-contain mb-4" />
    <h1 class="text-4xl mb-3 text-secondary-300 text-center font-title font-bold">
      Welcome to Omniminds Desktop
    </h1>
    <p class="text-lg text-gray-600">Train AI, Complete Tasks, Earn Rewards</p>
  </div>

  <div class="space-y-5">
    <p class="text-lg font-medium text-gray-700 mb-6">
      Just a couple of things before we get started:
    </p>

    <div class="space-y-8">
      <div class="bg-gray-50 p-6 rounded-xl border border-gray-100 shadow-sm">
        <h3 class="text-xl font-semibold text-gray-800 mb-4 flex items-center">
          <span
            class="bg-secondary-300 text-white rounded-full w-7 h-7 inline-flex items-center justify-center mr-3">
            1
          </span>
          Accept the Privacy Policy
        </h3>
        <div class="mb-4">
          <PrivacyPolicy class={'h-64'} />
        </div>
        <div class="flex items-center gap-3">
          <div class="flex items-center">
            <input
              type="checkbox"
              id="privacy-checkbox"
              class="w-5 h-5 text-secondary-300 bg-gray-100 border-gray-300 rounded focus:ring-secondary-300"
              bind:checked={$privacyAccepted} />
            <label for="privacy-checkbox" class="ml-2 text-gray-700">I accept</label>
          </div>
        </div>
      </div>

      <div class="bg-gray-50 p-6 rounded-xl border border-gray-100 shadow-sm">
        <h3 class="text-xl font-semibold text-gray-800 mb-4 flex items-center">
          <span
            class="bg-secondary-300 text-white rounded-full w-7 h-7 inline-flex items-center justify-center mr-3">
            2
          </span>
          Connect Your Wallet
        </h3>
        <div class="w-fit mx-auto mb-2">
          <WalletButton variant="large" theme="light" />
        </div>
        <p class="text-center text-sm text-gray-500 mt-2">
          {$walletAddress ? 'Wallet connected ✓' : 'Click to connect your wallet'}
        </p>
      </div>
    </div>

    <div class="flex justify-center mt-8 pt-4">
      <Button
        behavior="none"
        onclick={goNext}
        variant="green"
        disabled={!canContinue}
        class="px-8 py-3 text-lg font-medium">
        Continue
      </Button>
    </div>
  </div>
</div>
