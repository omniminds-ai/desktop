<script lang="ts">
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { onMount } from 'svelte';

  const { children } = $props();
</script>

<div class="relative">
  <div class="p-2 sm:p-4">
    <GymHeader />
    {@render children()}
  </div>
</div>
