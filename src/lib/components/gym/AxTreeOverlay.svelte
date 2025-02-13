<script lang="ts">
  export let tree: any[];
  export let videoElement: HTMLVideoElement | null;

  function renderAxTreeOverlay(tree: any) {
    return tree.map((node: any) => {
      const boxes = [];
      
      // Add this node's box if it has valid dimensions
      if (node.bbox && node.bbox.width > 0 && node.bbox.height > 0 && node.bbox.x > 0 && node.bbox.y > 0) {
        boxes.push({
          ...node.bbox,
          name: node.name,
          role: node.role
        });
      }
      
      // Recursively add children's boxes
      if (node.children && node.children.length > 0) {
        boxes.push(...renderAxTreeOverlay(node.children));
      }
      
      return boxes;
    }).flat();
  }

  $: boxes = renderAxTreeOverlay(tree);
  $: videoWidth = videoElement?.clientWidth || 0;
  $: videoHeight = videoElement?.clientHeight || 0;
  $: scale = Math.min(videoWidth / 3440, videoHeight / 1440);
</script>

<div 
  class="absolute inset-0 bg-black/50 text-white overflow-auto"
  style="width: {videoWidth}px; height: {videoHeight}px;"
>
  <div class="relative" style="transform: scale({scale}); transform-origin: top left;">
    {#each boxes as box}
      <div 
        class="absolute border border-white/50 flex items-center justify-center text-xs"
        style="left: {box.x}px; top: {box.y}px; width: {box.width}px; height: {box.height}px;"
        title="{box.name || 'Unnamed'} ({box.role})"
      >
        <span class="bg-black/75 px-1 truncate max-w-full">
          {box.name || box.role}
        </span>
      </div>
    {/each}
  </div>
</div>
