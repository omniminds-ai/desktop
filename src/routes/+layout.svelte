<script lang="ts">
  let { children } = $props();
</script>

<svelte:head>viralmind desktop</svelte:head>

{@render children()}

<style>
  :global(html, body) {
    background: transparent;
  }
</style>
