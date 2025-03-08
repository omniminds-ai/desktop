<script lang="ts">
  import { uploadManager } from '$lib/stores/misc';
  import Button from './Button.svelte';

  let { open, uploader }: { open: boolean; uploader: () => Promise<void> } = $props();
  const onCancel = async () => {
    open = false;
    await $uploadManager.setUploadDataAllowed(false);
  };
  const onConfirm = async () => {
    open = false;
    await $uploadManager.setUploadDataAllowed(true);
    await uploader();
  };
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="bg-white rounded-lg shadow-lg p-6 max-w-md w-full">
      <h2 class="text-xl font-semibold mb-4">Confirm Upload</h2>
      <p class="text-gray-700 mb-6">
        By uploading your demonstration data, you give Viralmind permission to process and train
        agents with it.
      </p>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={onCancel}>Cancel</Button>
        <Button variant="primary" onclick={onConfirm}>Confirm</Button>
      </div>
    </div>
  </div>
{/if}
