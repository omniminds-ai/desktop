<script lang="ts">
  import "../../app.css";
  
  let { children } = $props();
</script>

<div class="overlay-root">
  {@render children()}
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
  }

  .overlay-root {
    width: 100vw;
    height: 100vh;
    pointer-events: none;
    position: fixed;
    top: 0;
    left: 0;
    background: transparent !important;
  }
</style>
