<script lang="ts">
  const { children } = $props();
</script>

<div class="flex justify-center items-center min-h-screen p-8 bg-primary-600/75 backdrop-blur-md">
  {@render children()}
</div>
