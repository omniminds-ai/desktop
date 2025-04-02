<script lang="ts">
  import { Video } from 'lucide-svelte';
  import { onDestroy } from 'svelte';
  import Button from '$lib/components/form/Button.svelte';

  let countingDown = $state(false);
  let countdown = $state(3);
  let countdownInterval: number | undefined;

  const { onStart, onCancel, disabled } = $props<{
    onStart: () => void;
    onCancel: () => void;
    disabled: () => boolean;
  }>();

  function startCountdown() {
    if (countingDown || disabled() == true) return;

    countingDown = true;
    countdown = 3;

    countdownInterval = setInterval(() => {
      countdown--;
      if (countdown === 0) {
        clearInterval(countdownInterval);
        countdownInterval = undefined;
        countingDown = false;
        onStart();
      }
    }, 1000);
  }

  function cancelCountdown() {
    if (countdownInterval !== undefined) {
      clearInterval(countdownInterval);
      countdownInterval = undefined;
      countingDown = false;
      countdown = 3;
      onCancel();
    }
  }

  onDestroy(() => {
    if (countdownInterval !== undefined) {
      clearInterval(countdownInterval);
    }
  });
</script>

<Button
  disabled={disabled() == true}
  onclick={countingDown ? cancelCountdown : startCountdown}
  class={(countingDown ? 'bg-red-500! hover:bg-red-600! hover:text-white! border-red-500!' : '') +
    'rounded-md! flex! items-center gap-2'}>
  <Video size={20} />
  {#if countingDown}
    Cancel ({countdown}s)
  {:else}
    Start Recording Quest
  {/if}
</Button>
