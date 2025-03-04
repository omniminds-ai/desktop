<script lang="ts">
  export let minPrice = 1;
  export let maxPrice = 10;
  export let globalMin = 1;
  export let globalMax = 10;

  let sliderContainer: HTMLDivElement;
  let isDraggingMin = false;
  let isDraggingMax = false;
  let touchId: number | null = null;

  function getClientX(event: MouseEvent | TouchEvent): number {
    if (event instanceof MouseEvent) {
      return event.clientX;
    } else {
      const touch = Array.from(event.touches).find(t => t.identifier === touchId);
      return touch ? touch.clientX : event.touches[0].clientX;
    }
  }

  function updatePrice(clientX: number, isMin: boolean) {
    if (!sliderContainer) return;

    const rect = sliderContainer.getBoundingClientRect();
    // Account for padding in percentage calculation
    const padding = 16; // 4rem = 16px
    const percentage = Math.max(0, Math.min(1, (clientX - rect.left - padding) / (rect.width - padding * 2)));
    const range = globalMax - globalMin;
    const rawValue = globalMin + range * percentage;
    const value = Math.round(rawValue);

    if (isMin) {
      minPrice = Math.max(globalMin, Math.min(value, maxPrice - 1));
    } else {
      maxPrice = Math.max(minPrice + 1, Math.min(value, globalMax));
    }
  }

  function handleStart(event: MouseEvent | TouchEvent, isMin: boolean) {
    event.preventDefault();
    document.body.style.userSelect = 'none';
    
    if (event instanceof TouchEvent) {
      touchId = event.touches[0].identifier;
    }
    if (isMin) {
      isDraggingMin = true;
    } else {
      isDraggingMax = true;
    }
    updatePrice(getClientX(event), isMin);
  }

  function handleMove(event: MouseEvent | TouchEvent) {
    if (!isDraggingMin && !isDraggingMax) return;
    event.preventDefault();
    updatePrice(getClientX(event), isDraggingMin);
  }

  function handleEnd() {
    isDraggingMin = false;
    isDraggingMax = false;
    touchId = null;
    document.body.style.userSelect = '';
  }

  function handleInput(event: Event, isMin: boolean) {
    const input = event.target as HTMLInputElement;
    const value = parseInt(input.value) || (isMin ? globalMin : globalMax);
    if (isMin) {
      minPrice = Math.max(globalMin, Math.min(value, maxPrice - 1));
    } else {
      maxPrice = Math.max(minPrice + 1, Math.min(value, globalMax));
    }
  }

  // Ensure min/max prices stay within bounds
  $: {
    minPrice = Math.max(globalMin, Math.min(minPrice, maxPrice - 1));
    maxPrice = Math.max(minPrice + 1, Math.min(maxPrice, globalMax));
  }
</script>

<svelte:window 
  on:mousemove={handleMove} 
  on:mouseup={handleEnd}
  on:touchmove|preventDefault={handleMove}
  on:touchend={handleEnd}
  on:touchcancel={handleEnd}
/>

<div class="flex flex-col gap-1.5">
  <div class="text-sm font-medium text-gray-700">Choose your reward range</div>
  
  <!-- Price inputs -->
  <div class="flex items-center gap-2">
    <input
      type="number"
      min={globalMin}
      max={maxPrice - 1}
      class="w-16 px-2 py-1 text-sm border rounded hover:border-gray-400 focus:outline-none focus:border-secondary-300 focus:ring-1 focus:ring-secondary-300/30 bg-white transition-colors"
      value={minPrice}
      on:input={(e) => handleInput(e, true)}
    />
    <span class="text-sm text-gray-500">to</span>
    <input
      type="number"
      min={minPrice + 1}
      max={globalMax}
      class="w-16 px-2 py-1 text-sm border rounded hover:border-gray-400 focus:outline-none focus:border-secondary-300 focus:ring-1 focus:ring-secondary-300/30 bg-white transition-colors"
      value={maxPrice}
      on:input={(e) => handleInput(e, false)}
    />
    <span class="text-sm text-gray-500">VIRAL per task</span>
  </div>

  <!-- Slider container -->
  <div 
    class="relative h-8 px-4"
    bind:this={sliderContainer}
  >
    <!-- Track -->
    <div class="absolute inset-x-4 top-1/2 -translate-y-1/2">
      <!-- Background track -->
      <div class="absolute inset-x-0 h-1.5 bg-gray-200 rounded-full"></div>
      
      <!-- Selected range -->
      <div 
        class="absolute h-1.5 bg-secondary-300 rounded-full"
        style="left: calc({((minPrice - globalMin) / (globalMax - globalMin)) * 100}% + 0px); width: calc({((maxPrice - minPrice) / (globalMax - globalMin)) * 100}%)"
      ></div>
    </div>

    <!-- Slider handles -->
    <div class="absolute top-1/2 -translate-y-1/2 w-6 h-6 -translate-x-1/2 cursor-pointer touch-none z-10 group"
      style="left: calc(16px + ({((minPrice - globalMin) / (globalMax - globalMin))} * (100% - 32px)))"
      on:mousedown={(e) => handleStart(e, true)}
      on:touchstart={(e) => handleStart(e, true)}>
      <div class="absolute inset-0 bg-white border-2 border-secondary-300 rounded-full hover:scale-110 active:scale-95 transition-transform shadow-sm"></div>
      <div class="absolute -top-7 left-1/2 -translate-x-1/4 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-800 text-white text-xs rounded px-1.5 py-0.5 whitespace-nowrap shadow-sm pointer-events-none">
        {minPrice} VIRAL
      </div>
    </div>
    <div class="absolute top-1/2 -translate-y-1/2 w-6 h-6 -translate-x-1/2 cursor-pointer touch-none z-10 group"
      style="left: calc(16px + ({((maxPrice - globalMin) / (globalMax - globalMin))} * (100% - 32px)))"
      on:mousedown={(e) => handleStart(e, false)}
      on:touchstart={(e) => handleStart(e, false)}>
      <div class="absolute inset-0 bg-white border-2 border-secondary-300 rounded-full hover:scale-110 active:scale-95 transition-transform shadow-sm"></div>
      <div class="absolute -top-7 right-1/2 translate-x-1/4 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-800 text-white text-xs rounded px-1.5 py-0.5 whitespace-nowrap shadow-sm pointer-events-none">
        {maxPrice} VIRAL
      </div>
    </div>
  </div>
</div>
