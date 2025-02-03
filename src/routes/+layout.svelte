<script lang="ts">
  let { children } = $props();
</script>

{@render children()}

<style>
  :global(html, body) {
    background: transparent;
  }
</style>
