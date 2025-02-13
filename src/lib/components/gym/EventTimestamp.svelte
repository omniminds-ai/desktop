<script lang="ts">
  export let timestamp: number;
  export let startTime: number;
  export let videoElement: HTMLVideoElement | null = null;

  $: relativeTime = timestamp - startTime;
  $: formattedTime = formatTime(relativeTime);

  function formatTime(ms: number): string {
    const totalSeconds = ms / 1000;
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toFixed(3).padStart(6, '0')}`;
  }

  function handleClick() {
    if (videoElement) {
      const seconds = relativeTime / 1000;
      videoElement.currentTime = seconds;
    }
  }
</script>

<button
  class="px-2 py-0.5 text-sm text-gray-600 border border-gray-300 rounded hover:bg-gray-200 cursor-pointer transition-colors"
  on:click={handleClick}
  title="Click to seek video to this timestamp">
  {formattedTime}
</button>
