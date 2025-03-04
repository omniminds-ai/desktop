<script lang="ts">
  import Button from './Button.svelte';

  const { open, onConfirm, onCancel } = $props<{
    open: boolean;
    onConfirm: () => void;
    onCancel: () => void;
  }>();
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="bg-white rounded-lg shadow-lg p-6 max-w-md w-full">
      <h2 class="text-xl font-semibold mb-4">Confirm Upload</h2>
      <p class="text-gray-700 mb-6">
        By uploading your demonstration data, you give ViralmindAI permission to process and train
        agents with it.
      </p>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={onCancel}>Cancel</Button>
        <Button variant="primary" onclick={onConfirm}>Confirm</Button>
      </div>
    </div>
  </div>
{/if}
