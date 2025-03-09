<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Monitor, Cpu } from 'lucide-svelte';
  import { startEngine, getEngineStatus } from '$lib/stores/engine';
  import type { EngineStatus } from '$lib/stores/engine';

  let engineStatus = $state<EngineStatus>({
    running: false,
    system_info: null,
    gpu_info: null,
    error: null
  });

  // Function to get a shorter version of system info
  function formatSystemInfo(info: string | null): string {
    if (!info) return 'Unknown OS';
    
    // Windows
    if (info.includes('Microsoft Windows')) {
      return info.split('[')[0].trim();
    }
    
    // Linux
    if (info.includes('Linux')) {
      const parts = info.split(' ');
      return `${parts[0]} ${parts[2]}`;
    }
    
    // macOS
    if (info.includes('Darwin')) {
      return `macOS ${info.match(/\d+\.\d+\.\d+/)?.[0] || ''}`;
    }
    
    return info.substring(0, 30) + (info.length > 30 ? '...' : '');
  }
  
  // Function to get GPU model from the info
  function formatGpuInfo(info: string | null): string {
    if (!info) return 'No GPU detected';
    if (info.includes('No GPU info available')) return 'No GPU detected';
    
    // Extract NVIDIA GPU model if present
    const nvidiaMatch = info.match(/NVIDIA GeForce ([^\s]+)/);
    if (nvidiaMatch) return `NVIDIA ${nvidiaMatch[1]}`;
    
    // Extract AMD GPU model if present
    const amdMatch = info.match(/AMD Radeon ([^\s]+)/);
    if (amdMatch) return `AMD ${amdMatch[1]}`;
    
    // Extract Intel GPU model if present
    const intelMatch = info.match(/Intel ([^\s]+) Graphics/);
    if (intelMatch) return `Intel ${intelMatch[1]}`;
    
    return 'GPU detected';
  }
  
  // Listen for engine status updates
  let unsubscribe: (() => void) | null = null;
  
  async function fetchEngineStatus() {
    try {
      engineStatus = await getEngineStatus();
    } catch (error) {
      console.error('Failed to get engine status:', error);
    }
  }
  
  let interval: ReturnType<typeof setInterval>;
  
  onMount(() => {
    // Start the engine and get initial status
    // startEngine().catch(console.error);
    fetchEngineStatus();
    
    // Subscribe to status updates
    (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unsubscribe = await listen('engine-status', (event) => {
          engineStatus = event.payload as any;
        });
      } catch (error) {
        console.error('Failed to subscribe to engine status events:', error);
      }
    })();
    
    // Poll for status updates every 10 seconds as a fallback
    interval = setInterval(fetchEngineStatus, 10000);
    
    return () => {
      clearInterval(interval);
      if (unsubscribe) unsubscribe();
    };
  });
  
  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });
</script>

{#if engineStatus.running}
  <div class="status-bar bg-secondary-100/25">
    <div class="status-item">
      <Monitor size={16} />
      <span>{formatSystemInfo(engineStatus.system_info)}</span>
    </div>
    <div class="status-item">
      <Cpu size={16} />
      <span>{formatGpuInfo(engineStatus.gpu_info)}</span>
    </div>
  </div>
{/if}

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 4px 12px;
    /* background-color: rgba(0, 0, 0, 0.25); */
    color: white;
    font-size: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .status-item {
    display: flex;
    align-items: center;
    gap: 6px;
    opacity: 0.8;
  }
  
  .status-bar :global(svg) {
    stroke-width: 1.5;
  }
</style>
