<script lang="ts">
  import { toasts } from '$lib/stores/toast';
  import { onMount } from 'svelte';
  import Toast from './Toast.svelte';
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col items-end pointer-events-none">
  {#each $toasts as toast (toast.id)}
    <div class="pointer-events-auto">
      <Toast {toast} />
    </div>
  {/each}
</div>
