<script lang="ts">
  import type { Token, TokenType } from '$lib/types/forge';
  import { fade } from 'svelte/transition';
  import Button from './Button.svelte';
  import Input from './Input.svelte';
  import TextArea from './TextArea.svelte';

  const {
    show,
    onClose,
    onCreate
  }: {
    show: boolean;
    onClose: () => void;
    onCreate: (data: { name: string; skills: string; token: Token }) => void;
  } = $props();

  let name = $state('');
  let skills = $state('');
  let selectedTokenType: TokenType = $state('VIRAL');
  let customTokenAddress = $state('');

  const DEFAULT_ADDRESSES = {
    SOL: 'So11111111111111111111111111111111111111112',
    VIRAL: 'ViRA1111111111111111111111111111111111111112'
  };

  const tokenOptions: { type: TokenType; symbol: string }[] = [
    { type: 'VIRAL', symbol: 'VIRAL' },
    { type: 'SOL', symbol: 'SOL' },
    { type: 'CUSTOM', symbol: 'Custom Token' }
  ];

  function handleSubmit() {
    const token: Token =
      selectedTokenType === 'CUSTOM'
        ? { type: 'CUSTOM', symbol: 'SOL', address: customTokenAddress }
        : {
            type: selectedTokenType,
            symbol: selectedTokenType,
            address: DEFAULT_ADDRESSES[selectedTokenType]
          };

    onCreate({ name, skills, token });
    resetForm();
  }

  function handleClose() {
    resetForm();
    onClose();
  }

  function resetForm() {
    name = '';
    skills = '';
    selectedTokenType = 'SOL';
    customTokenAddress = '';
  }
</script>

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    transition:fade={{ duration: 100 }}>
    <div class="bg-gray-900 rounded-xl w-full max-w-lg p-6 space-y-4">
      <h2 class="text-xl font-bold text-white">Create Training Pool</h2>

      <form onsubmit={handleSubmit} class="space-y-4">
        <div>
          <label for="pool-name" class="block text-sm font-semibold text-gray-300 mb-1">
            Pool Name
          </label>
          <Input
            id="pool-name"
            type="text"
            bind:value={name}
            class="w-full p-2 bg-gray-800 rounded-lg text-white border border-gray-700 focus:border-secondary-300 focus:outline-none"
            placeholder="Enter pool name..."
            required />
        </div>

        <div>
          <label for="skill-list" class="block text-sm font-semibold text-gray-300 mb-1">
            Skills to Collect
          </label>
          <TextArea
            id="skill-list"
            bind:value={skills}
            placeholder="List the skills you want to collect demonstrations for..."
            required>
          </TextArea>
        </div>

        <div>
          <span class="block text-sm font-semibold text-gray-300 mb-1">Treasury Token</span>
          <div class="space-y-2">
            {#each tokenOptions as option}
              <label class="flex items-center gap-2 text-gray-300">
                <input
                  type="radio"
                  bind:group={selectedTokenType}
                  value={option.type}
                  class="text-secondary-300 focus:ring-secondary-300" />
                <span>{option.symbol}</span>
              </label>
            {/each}

            {#if selectedTokenType === 'CUSTOM'}
              <Input
                type="text"
                bind:value={customTokenAddress}
                placeholder="Enter token contract address..."
                required />
            {/if}
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-4">
          <Button
            type="button"
            class="hover:bg-gray-800! hover:text-white! py-2!"
            onclick={handleClose}>
            Cancel
          </Button>
          <Button type="submit" class="hover:bg-gray-800! hover:text-white! py-2!">
            Create Pool
          </Button>
        </div>
      </form>
    </div>
  </div>
{/if}
