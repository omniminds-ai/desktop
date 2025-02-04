<script lang="ts">
  import { Video } from "lucide-svelte";
  import { onDestroy } from "svelte";

  let countingDown = $state(false);
  let countdown = $state(3);
  let countdownInterval: number | undefined;

  const { onStart, onCancel } = $props<{
    onStart: () => void;
    onCancel: () => void;
  }>();

  function startCountdown() {
    if (countingDown) return;

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

<button
  onclick={countingDown ? cancelCountdown : startCountdown}
  class="px-4 py-2 bg-secondary-300 text-white rounded hover:bg-secondary-400 hover:shadow-md transition-all duration-200 tracking-wide font-medium flex items-center gap-2"
  class:bg-red-500={countingDown}
  class:hover:bg-red-700={countingDown}
>
  <Video size={16} />
  {#if countingDown}
    Cancel ({countdown}s)
  {:else}
    Start Recording Quest
  {/if}
</button>
