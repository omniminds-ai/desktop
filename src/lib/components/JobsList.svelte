<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { writable } from 'svelte/store';
  import { Cpu, Layers, ChevronDown, ChevronUp, Terminal, ChevronRight, Trash2 } from 'lucide-svelte';

  export let expanded = false;
  let selectedJobId: string | null = null;

  // Define the job structure based on what we saw in engine.rs
  interface CommandJob {
    id: string;
    command: string;
    status: 'Running' | 'Success' | 'Failed';
    output: string;
    error: string;
  }

  // Track seen jobs
  const jobs = writable<CommandJob[]>([]);
  const seenJobIds = new Set<string>();
  let unlistenFunctions: Array<() => void> = [];
  let unseenOrRunningCount = 0;

  function updateUnseenOrRunningCount(allJobs: CommandJob[]) {
    // Count jobs that are either unseen or still running
    unseenOrRunningCount = allJobs.filter(job => 
      job.status === 'Running' || !seenJobIds.has(job.id)
    ).length;
  }

  async function fetchJobs() {
    try {
      const allJobs: CommandJob[] = await invoke('get_all_jobs');
      jobs.set(allJobs);
      updateUnseenOrRunningCount(allJobs);
    } catch (error) {
      console.error('Failed to fetch jobs:', error);
    }
  }

  onMount(async () => {
    // Initial fetch of jobs
    await fetchJobs();

    // Set up regular polling for jobs
    const intervalId = setInterval(fetchJobs, 2000);

    // Listen for job updates
    const unlisten1 = await listen('job-created', (event) => {
      const newJob = event.payload as CommandJob;
      jobs.update(currentJobs => {
        const updatedJobs = [...currentJobs, newJob];
        updateUnseenOrRunningCount(updatedJobs);
        return updatedJobs;
      });
    });

    const unlisten2 = await listen('job-updated', (event) => {
      const updatedJob = event.payload as CommandJob;
      jobs.update(currentJobs => {
        const updatedJobs = currentJobs.map(job => job.id === updatedJob.id ? updatedJob : job);
        updateUnseenOrRunningCount(updatedJobs);
        return updatedJobs;
      });
    });
    
    // Listen for jobs-cleared event from the backend
    const unlisten3 = await listen('jobs-cleared', () => {
      // Update our UI to only show running jobs
      jobs.update(currentJobs => currentJobs.filter(job => job.status === 'Running'));
      
      // Clear selection if it was a non-running job
      if (selectedJobId) {
        const selectedJob = $jobs.find(job => job.id === selectedJobId);
        if (!selectedJob || selectedJob.status !== 'Running') {
          selectedJobId = null;
        }
      }
      
      updateUnseenOrRunningCount($jobs);
    });

    unlistenFunctions.push(unlisten1, unlisten2, unlisten3);
    
    // Add the interval clearing function to cleanup
    unlistenFunctions.push(() => clearInterval(intervalId));
  });

  onDestroy(() => {
    // Clean up event listeners and intervals
    unlistenFunctions.forEach(unlisten => unlisten());
  });

  function toggleExpanded() {
    expanded = !expanded;
    
    // Mark all jobs as seen when the panel is expanded
    if (expanded) {
      $jobs.forEach(job => {
        seenJobIds.add(job.id);
      });
      updateUnseenOrRunningCount($jobs);
    }
  }

  function getStatusClass(status: string) {
    switch (status) {
      case 'Running': return 'bg-yellow-500';
      case 'Success': return 'bg-green-500';
      case 'Failed': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  }

  // Function to clear finished jobs
  async function clearFinishedJobs() {
    try {
      // Call the Tauri command to clear finished jobs (non-running jobs)
      await invoke('clear_all_jobs');
      
      // Update the UI to show only running jobs
      jobs.update(currentJobs => currentJobs.filter(job => job.status === 'Running'));
      
      // If we had a non-running job selected, clear the selection
      if (selectedJobId) {
        const selectedJob = $jobs.find(job => job.id === selectedJobId);
        if (!selectedJob || selectedJob.status !== 'Running') {
          selectedJobId = null;
        }
      }
      
      // Update the unseen count (should only include running jobs now)
      updateUnseenOrRunningCount($jobs);
    } catch (error) {
      console.error('Failed to clear jobs:', error);
    }
  }

  function formatJobName(command: string) {
    // Extract filename for curl commands (downloads)
    if (command.includes('curl -L')) {
      try {
        // Match the output filename
        const match = command.match(/-o\s+"([^"]+)\/([^"/]+)"/);
        if (match && match[2]) {
          return `Downloading ${match[2]}`;
        }
      } catch (e) {
        // If regex fails, just return a generic name
      }
      return 'Downloading file...';
    }
    
    // For other commands, just return a shortened version
    return command.length > 40 ? command.substring(0, 37) + '...' : command;
  }
</script>

<div class="w-full">
  <button
    onclick={toggleExpanded}
    class="w-full py-2 flex justify-center rounded-full transition-colors hover:bg-white/10 text-gray-300 relative"
    title="Jobs">
    <Cpu size={20} />
    
    {#if unseenOrRunningCount > 0}
      <div class="absolute -top-1 -right-1 w-4 h-4 bg-secondary-300 rounded-full flex items-center justify-center text-[10px] text-white font-bold">
        {unseenOrRunningCount}
      </div>
    {/if}
  </button>

  <!-- Jobs Popup -->
  {#if expanded}
    <div class="invisible !visible opacity-100 transition-all duration-200
      absolute left-16 bottom-6 z-50 w-96 h-96
      bg-primary-600 text-gray-300 text-xs rounded-lg shadow-2xl border border-primary-600 flex flex-col">
        <div class="flex justify-between items-center px-3 py-2 border-b border-primary-500">
          <h3 class="font-semibold text-sm">Engine Jobs</h3>
          <div class="flex items-center gap-2">
            <button 
              onclick={clearFinishedJobs} 
              class="text-gray-400 hover:text-white text-sm p-1 rounded hover:bg-white/10"
              title="Clear completed jobs"
            >
              <Trash2 size={14} />
            </button>
            <button onclick={toggleExpanded} class="text-gray-400 hover:text-white text-sm">&times;</button>
          </div>
        </div>
        
          {#if $jobs.length === 0}
            <div class="flex-1 flex items-center justify-center text-gray-400 p-3">No active jobs</div>
          {:else if selectedJobId}
            <!-- Mark the selected job as seen -->
            {seenJobIds.add(selectedJobId) && ''}
          {#each $jobs as job}
            {#if job.id === selectedJobId}
              <div class="flex-1 flex flex-col overflow-hidden p-3">
                <div class="flex justify-between items-center mb-2">
                  <div class="flex items-center gap-2">
                    <button 
                      class="p-1 rounded hover:bg-gray-700 transition-colors" 
                      onclick={() => selectedJobId = null}
                      title="Back to job list"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                    </button>
                    <div class="flex items-center gap-1">
                      <span class="inline-block w-2 h-2 rounded-full {getStatusClass(job.status)}"></span>
                      <span class="font-medium">{formatJobName(job.command)}</span>
                    </div>
                  </div>
                  <div class="text-xs text-gray-400">
                    {job.status}
                  </div>
                </div>
                
                <div class="flex-1 overflow-y-auto bg-primary-700 rounded p-2 font-mono text-xs whitespace-pre-wrap">
                  {#if job.status === 'Running'}
                    <div class="text-yellow-400">
                      Job is still running...
                    </div>
                  {:else if job.status === 'Success' && job.output}
                    <div class="text-green-400">
                      {job.output}
                    </div>
                  {:else if job.status === 'Failed' && job.error}
                    <div class="text-red-400">
                      {job.error}
                    </div>
                  {:else}
                    <div class="text-gray-400">
                      No output available
                    </div>
                  {/if}
                </div>
                
                <div class="mt-2 text-gray-300 text-xs">
                  <div><span class="font-semibold">ID:</span> {job.id}</div>
                  <div class="mt-1"><span class="font-semibold">Command:</span></div>
                  <div class="font-mono text-xs bg-primary-700 rounded p-2 whitespace-pre-wrap mt-1 max-h-24 overflow-y-auto">{job.command}</div>
                </div>
              </div>
            {/if}
          {/each}
        {:else}
          <div class="flex-1 overflow-y-auto p-3">
            {#each $jobs as job}
              <button 
                class="mb-2 p-2 w-full bg-primary-700 rounded-md text-xs hover:bg-white/5 transition-colors text-left"
                onclick={() => {
                  selectedJobId = job.id;
                  seenJobIds.add(job.id);
                  updateUnseenOrRunningCount($jobs);
                }}
              >
                <div class="flex justify-between items-center">
                  <div class="flex items-center">
                    <span class="inline-block w-2 h-2 mr-2 rounded-full {getStatusClass(job.status)}"></span>
                    <div class="truncate flex-1 max-w-52" title={job.command}>
                      {formatJobName(job.command)}
                    </div>
                  </div>
                  <div class="ml-2 text-gray-400">
                    <ChevronRight size={14} />
                  </div>
                </div>
                
                {#if job.status === 'Running'}
                  <div class="mt-2 w-full h-1 bg-primary-500 rounded-full overflow-hidden">
                    <div class="h-full bg-secondary-300 rounded-full animate-pulse"></div>
                  </div>
                {:else if job.status === 'Success' && job.output}
                  <div class="mt-1 text-green-400 text-xs truncate" title={job.output}>
                    {job.output.includes('Downloaded to') ? job.output : 'Completed'}
                  </div>
                {:else if job.status === 'Failed' && job.error}
                  <div class="mt-1 text-red-400 text-xs truncate" title={job.error}>
                    Error: {job.error}
                  </div>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
  {/if}
</div>
