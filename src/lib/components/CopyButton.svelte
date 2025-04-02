<script lang="ts">
  import Button from '$lib/components/form/Button.svelte';
  let copied = $state(false);

  const { content }: { content: string } = $props();

  const copy = () => {
    navigator.clipboard.writeText(content);
    copied = true;
    setTimeout(() => (copied = false), 750);
  };
</script>

<Button variant="secondary" class="px-3 py-1.5 text-sm w-20" onclick={copy}>
  {#if copied}
    Copied!
  {:else}
    Copy
  {/if}
</Button>
