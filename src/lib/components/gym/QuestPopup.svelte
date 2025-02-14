<script lang="ts">
  import AppText from './AppText.svelte';
  
  export let title: string;
  export let objectives: string[];
  export let onAccept: () => void;
  export let onDecline: () => void;
</script>

<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
  <div class="bg-zinc-900 rounded-lg p-6 max-w-md w-full mx-4 border border-zinc-700 shadow-xl">
    <div class="mb-6">
      <h2 class="text-2xl font-bold text-white mb-2">New Quest Available!</h2>
      <h3 class="text-xl text-white mb-4">{title}</h3>
      
      <div class="mb-6">
        <h4 class="text-lg font-semibold text-white mb-2">Quest Objectives</h4>
        <ul class="list-disc pl-5 space-y-2">
          {#each objectives as objective}
            <li class="text-zinc-300">
              <AppText text={objective} />
            </li>
          {/each}
        </ul>
      </div>
    </div>

    <div class="flex gap-4">
      <button
        on:click={onAccept}
        class="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white font-bold py-2 px-4 rounded transition-colors"
      >
        Accept Quest
      </button>
      <button
        on:click={onDecline}
        class="flex-1 bg-zinc-700 hover:bg-zinc-600 text-white font-bold py-2 px-4 rounded transition-colors"
      >
        Decline
      </button>
    </div>
  </div>
</div>
