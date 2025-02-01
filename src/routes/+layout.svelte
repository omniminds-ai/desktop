<script lang="ts">
  import Sidebar from '$lib/components/Sidebar.svelte';
  import "../app.css";
  
  let { children } = $props();
  
  </script>
  
  <div class="h-screen flex bg-transparent overflow-hidden">
    <Sidebar />
    <div class="flex-1 p-2">
      <div class="h-full overflow-hidden bg-gray-50 border border-gray-500 rounded-md">
        <div class="h-full w-full overflow-auto">
          {@render children()}
        </div>
      </div>
    </div>
  </div>
  
  <style>
    :global(html, body) {
      background: transparent;
    }
  </style>
  