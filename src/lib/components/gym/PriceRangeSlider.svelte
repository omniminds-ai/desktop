<script lang="ts">
  // Default values
  const DEFAULT_MIN = 0;
  const DEFAULT_MAX = 500;
  type AppTouchEvent = TouchEvent;

  export let minPrice = DEFAULT_MIN;
  export let maxPrice = DEFAULT_MAX;
  export let globalMin = DEFAULT_MIN;
  export let globalMax = DEFAULT_MAX;

  let sliderContainer: HTMLDivElement;
  let isDraggingMin = false;
  let isDraggingMax = false;
  let touchId: number | null = null;

  // Normalize price values to handle NaN or invalid values
  function normalize(value: number, defaultValue: number): number {
    return isNaN(value) || !isFinite(value) ? defaultValue : value;
  }

  // Get safe values for calculations
  function safeMinPrice(): number {
    return normalize(minPrice, DEFAULT_MIN);
  }

  function safeMaxPrice(): number {
    return normalize(maxPrice, DEFAULT_MAX);
  }

  function safeGlobalMin(): number {
    return normalize(globalMin, DEFAULT_MIN);
  }

  function safeGlobalMax(): number {
    return normalize(globalMax, DEFAULT_MAX);
  }

  function getClientX(event: MouseEvent | AppTouchEvent): number {
    if (event instanceof MouseEvent) {
      return event.clientX;
    } else {
      const touch = Array.from(event.touches).find((t) => t.identifier === touchId);
      return touch ? touch.clientX : event.touches[0].clientX;
    }
  }

  function updatePrice(clientX: number, isMin: boolean) {
    if (!sliderContainer) return;

    const rect = sliderContainer.getBoundingClientRect();
    // Account for padding in percentage calculation
    const padding = 16; // 4rem = 16px
    const width = Math.max(1, rect.width - padding * 2); // Ensure positive width
    const percentage = Math.max(0, Math.min(1, (clientX - rect.left - padding) / width));

    const min = safeGlobalMin();
    const max = safeGlobalMax();
    const range = Math.max(1, max - min); // Ensure positive range

    const rawValue = min + range * percentage;
    const value = Math.round(rawValue);

    if (isMin) {
      minPrice = Math.max(min, Math.min(value, safeMaxPrice() - 1));
    } else {
      maxPrice = Math.max(safeMinPrice() + 1, Math.min(value, max));
    }
  }

  function handleStart(event: MouseEvent | AppTouchEvent, isMin: boolean) {
    event.preventDefault();
    document.body.style.userSelect = 'none';

    // some stupid safari polyfill
    if ('touches' in event) {
      touchId = event.touches[0].identifier;
    }
    if (isMin) {
      isDraggingMin = true;
    } else {
      isDraggingMax = true;
    }
    updatePrice(getClientX(event), isMin);
  }

  function handleMove(event: MouseEvent | AppTouchEvent) {
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
    let value = parseInt(input.value);

    // Handle NaN
    if (isNaN(value)) {
      value = isMin ? safeGlobalMin() : safeGlobalMax();
    }

    if (isMin) {
      minPrice = Math.max(safeGlobalMin(), Math.min(value, safeMaxPrice() - 1));
    } else {
      maxPrice = Math.max(safeMinPrice() + 1, Math.min(value, safeGlobalMax()));
    }
  }

  // Ensure min/max prices stay within bounds and handle NaN
  $: {
    const sMin = safeGlobalMin();
    const sMax = safeGlobalMax();
    const sMinPrice = safeMinPrice();
    const sMaxPrice = safeMaxPrice();

    // Make sure minPrice is valid
    minPrice = Math.max(sMin, Math.min(sMinPrice, sMaxPrice - 1));

    // Make sure maxPrice is valid
    maxPrice = Math.max(minPrice + 1, Math.min(sMaxPrice, sMax));
  }

  // Calculate safe percentage values for the UI
  function getMinPercent(): number {
    const min = safeGlobalMin();
    const max = safeGlobalMax();
    const range = max - min;
    return range <= 0 ? 0 : ((safeMinPrice() - min) / range) * 100;
  }

  function getMaxPercent(): number {
    const min = safeGlobalMin();
    const max = safeGlobalMax();
    const range = max - min;
    return range <= 0 ? 100 : ((safeMaxPrice() - min) / range) * 100;
  }

  function getRangeWidth(): number {
    return Math.max(0, getMaxPercent() - getMinPercent());
  }
</script>

<svelte:window
  on:mousemove={handleMove}
  on:mouseup={handleEnd}
  on:touchmove|preventDefault={handleMove}
  on:touchend={handleEnd}
  on:touchcancel={handleEnd} />

<div class="flex flex-col gap-1.5">
  <div class="text-sm font-medium text-gray-700">Choose your reward range</div>

  <!-- Price inputs -->
  <div class="flex items-center gap-2">
    <input
      type="number"
      min={globalMin || 0}
      max={maxPrice - 1 || 499}
      class="w-16 px-2 py-1 text-sm border rounded hover:border-gray-400 focus:outline-none focus:border-secondary-300 focus:ring-1 focus:ring-secondary-300/30 bg-white transition-colors"
      value={minPrice || 0}
      on:input={(e) => handleInput(e, true)} />
    <span class="text-sm text-gray-500">to</span>
    <input
      type="number"
      min={minPrice + 1 || 1}
      max={globalMax || 500}
      class="w-16 px-2 py-1 text-sm border rounded hover:border-gray-400 focus:outline-none focus:border-secondary-300 focus:ring-1 focus:ring-secondary-300/30 bg-white transition-colors"
      value={maxPrice || 500}
      on:input={(e) => handleInput(e, false)} />
    <span class="text-sm text-gray-500">VIRAL per task</span>
  </div>

  <!-- Slider container -->
  <div class="relative h-8 px-4" bind:this={sliderContainer}>
    <!-- Track -->
    <div class="absolute inset-x-4 top-1/2 -translate-y-1/2">
      <!-- Background track -->
      <div class="absolute inset-x-0 h-1.5 bg-gray-200 rounded-full"></div>

      <!-- Selected range -->
      <div
        class="absolute h-1.5 bg-secondary-300 rounded-full"
        style="left: {getMinPercent()}%; width: {getRangeWidth()}%">
      </div>
    </div>

    <!-- Slider handles -->
    <div
      role="slider"
      aria-valuenow={safeMinPrice()}
      tabindex="0"
      class="absolute top-1/2 -translate-y-1/2 w-6 h-6 -translate-x-1/2 cursor-pointer touch-none z-10 group"
      style="left: calc(16px + ({getMinPercent() / 100} * (100% - 32px)))"
      on:mousedown={(e) => handleStart(e, true)}
      on:touchstart={(e) => handleStart(e, true)}>
      <div
        class="absolute inset-0 bg-white border-2 border-secondary-300 rounded-full hover:scale-110 active:scale-95 transition-transform shadow-sm">
      </div>
      <div
        class="absolute -top-7 left-1/2 -translate-x-1/4 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-800 text-white text-xs rounded px-1.5 py-0.5 whitespace-nowrap shadow-sm pointer-events-none">
        {safeMinPrice()} VIRAL
      </div>
    </div>
    <div
      role="slider"
      aria-valuenow={safeMaxPrice()}
      tabindex="0"
      class="absolute top-1/2 -translate-y-1/2 w-6 h-6 -translate-x-1/2 cursor-pointer touch-none z-10 group"
      style="left: calc(16px + ({getMaxPercent() / 100} * (100% - 32px)))"
      on:mousedown={(e) => handleStart(e, false)}
      on:touchstart={(e) => handleStart(e, false)}>
      <div
        class="absolute inset-0 bg-white border-2 border-secondary-300 rounded-full hover:scale-110 active:scale-95 transition-transform shadow-sm">
      </div>
      <div
        class="absolute -top-7 right-1/2 translate-x-1/4 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-800 text-white text-xs rounded px-1.5 py-0.5 whitespace-nowrap shadow-sm pointer-events-none">
        {safeMaxPrice()} VIRAL
      </div>
    </div>
  </div>
</div>
