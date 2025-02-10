<script lang="ts">
  import type { Token, TokenType } from '$lib/types/forge';

  export let show = false;
  export let onClose: () => void;
  export let onCreate: (data: { name: string; skills: string; token: Token }) => void;

  let name = '';
  let skills = '';
  let selectedTokenType: TokenType = 'VIRAL';
  let customTokenAddress = '';

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
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-gray-900 rounded-xl w-full max-w-lg p-6 space-y-4">
      <h2 class="text-xl font-bold text-white">Create Training Pool</h2>

      <form on:submit|preventDefault={handleSubmit} class="space-y-4">
        <div>
          <label for="pool-name" class="block text-sm font-semibold text-gray-300 mb-1">
            Pool Name
          </label>
          <input
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
          <textarea
            id="skill-list"
            bind:value={skills}
            class="w-full h-32 p-2 bg-gray-800 rounded-lg text-white border border-gray-700 focus:border-secondary-300 focus:outline-none resize-none"
            placeholder="List the skills you want to collect demonstrations for..."
            required>
          </textarea>
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
              <input
                type="text"
                bind:value={customTokenAddress}
                class="w-full p-2 bg-gray-800 rounded-lg text-white border border-gray-700 focus:border-secondary-300 focus:outline-none mt-2"
                placeholder="Enter token contract address..."
                required />
            {/if}
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-4">
          <button
            type="button"
            class="px-4 py-2 text-gray-300 hover:text-white transition-colors"
            on:click={handleClose}>
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-secondary-300 text-white rounded-lg hover:bg-secondary-400 transition-colors">
            Create Pool
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
