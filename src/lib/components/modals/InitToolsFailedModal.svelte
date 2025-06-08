<script lang="ts">
  import Button from '$lib/components/form/Button.svelte';

  let {
    open = $bindable(),
    retry,
    errors = []
  }: { open: boolean; errors: string[]; retry: () => Promise<void> } = $props();
  const onCancel = async () => {
    open = false;
  };
  const onConfirm = async () => {
    open = false;
    await retry();
  };
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="bg-white rounded-lg shadow-lg p-6 max-w-md w-full">
      <h2 class="text-xl font-semibold mb-4">Tool Installation Failed</h2>
      <p class="text-gray-700 mb-2">
        An error occured when downloading and installing tools required to run Omniminds Desktop.
      </p>
      <p class="text-gray-700 mb-6">Please retry.</p>
      <div class="bg-gray-100 rounded-lg shadow-md p-4 my-2">
        <pre
          class="overflow-auto whitespace-pre">{#each errors.map((e) => e.trim()) as error}{error +
              '\n'}{/each}</pre>
      </div>
      <p class="text-gray-700 text-sm mt-1 mb-5">
        If this keeps happening, create a support thread in our <a
          href="https://discord.gg/C9NyQmkz5W"
          class="hover:underline text-secondary-300">
          Discord.
        </a>
      </p>
      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={onCancel}>Ignore</Button>
        <Button variant="primary" onclick={onConfirm}>Retry</Button>
      </div>
    </div>
  </div>
{/if}
