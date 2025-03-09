<script lang="ts">
  import Card from '../Card.svelte';
  import Button from '../Button.svelte';
  import DownloadScriptsModal from '../DownloadScriptsModal.svelte';
  import { Code, Download, Clock, Monitor, Laptop, Server, Smartphone, Globe, Languages } from 'lucide-svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import { getPoolSubmissions, type PoolSubmission } from '$lib/api/forge';
  import { onMount } from 'svelte';

  export let pool: TrainingPool & {
    unsavedSkills?: boolean;
    unsavedPrice?: boolean;
    unsavedName?: boolean;
  };

  let submissions: PoolSubmission[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      loading = true;
      error = null;
      submissions = await getPoolSubmissions(pool._id);
    } catch (err) {
      console.error('Failed to fetch pool submissions:', err);
      error = 'Failed to load submissions. Please try again later.';
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  function getTitle(submission: PoolSubmission): string {
    if (submission.meta?.quest?.title) {
      return submission.meta.quest.title;
    } else if (submission.meta?.quest?.app) {
      return submission.meta.quest.app;
    }
    return 'Untitled Submission';
  }

  function getTotalFileSize(submission: PoolSubmission): string {
    if (!submission.files || submission.files.length === 0) {
      return '-';
    }
    
    let totalSize = 0;
    let hasSizeInfo = false;
    
    submission.files.forEach(file => {
      if (file.size) {
        totalSize += file.size;
        hasSizeInfo = true;
      }
    });
    
    if (!hasSizeInfo) {
      return '-';
    }
    
    // Format file size in MB
    return `${(totalSize / (1024 * 1024)).toFixed(2)} MB`;
  }

  function getLocaleCode(submission: PoolSubmission): string {
    const locale = submission.meta?.locale || '';
    return locale;
  }
  
  function getOSIcon(submission: PoolSubmission): any {
    const platform = submission.meta?.platform?.toLowerCase() || '';
    
    if (platform.includes('windows')) return Monitor;
    if (platform.includes('mac') || platform.includes('darwin')) return Laptop; 
    if (platform.includes('linux')) return Server;
    if (platform.includes('android') || platform.includes('ios')) return Smartphone;
    
    return Globe;
  }

  function getPlatformLabel(submission: PoolSubmission): string {
    const platform = submission.meta?.platform?.toLowerCase() || '';
    return platform;
  }
  
  function formatDuration(submission: PoolSubmission): string {
    const seconds = submission.meta?.duration_seconds;
    if (!seconds && seconds !== 0) return '-';
    
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    
    if (minutes > 0) {
      return `${minutes}m ${remainingSeconds}s`;
    } else {
      return `${remainingSeconds}s`;
    }
  }

  // Download scripts modal
  let showDownloadModal = false;
</script>

<!-- Header section similar to TasksTab -->
<div class="flex justify-between items-center border-gray-200 pb-3">
  <div class="flex items-center">
    <h2 class="text-xl font-bold text-gray-800">Uploads</h2>
  </div>
  <div class="flex items-center">
    <button
      class="px-3 py-1.5 text-sm rounded-md bg-secondary-300 text-white hover:bg-secondary-100 transition-colors flex items-center"
      on:click={() => (showDownloadModal = true)}
      disabled={submissions.length === 0}>
      <Code size={15} class="mr-1" />
      Export All Uploads
    </button>
  </div>
</div>

<Card padding="none" className="border border-gray-200">
  {#if loading}
    <div class="p-8 text-center text-gray-500">
      <div class="animate-pulse flex flex-col items-center">
        <div class="text-sm font-medium mb-2">Loading submissions...</div>
        <div class="bg-gray-200 h-2 w-40 rounded"></div>
      </div>
    </div>
  {:else if error}
    <div class="p-8 text-center text-red-500">
      <div class="text-sm font-medium">{error}</div>
      <button
        class="mt-2 text-secondary-600 hover:text-secondary-900 text-sm hover:underline cursor-pointer"
        on:click={() => {
          loading = true;
          getPoolSubmissions(pool._id)
            .then((data) => {
              submissions = data;
              error = null;
            })
            .catch((err) => {
              console.error('Failed to reload submissions:', err);
              error = 'Failed to load submissions. Please try again later.';
            })
            .finally(() => {
              loading = false;
            });
        }}>
        Try Again
      </button>
    </div>
  {:else}
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="w-16 px-2 py-3"></th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Task
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Status
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Duration
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            File Size
          </th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each submissions as submission}
          <tr>
            <td class="px-2 py-4 whitespace-nowrap text-gray-500">
              <div class="flex items-center gap-2">
                <!-- <span class="flex items-center gap-1 bg-gray-200 text-gray-700 px-1.5 py-0.5 rounded text-xs font-mono">
                  <Languages size={10} class="text-gray-500" />
                  {getLocaleCode(submission)}
                </span> -->
                <span class="flex items-center gap-1 bg-gray-200 text-gray-700 px-1.5 py-0.5 rounded text-xs font-mono">
                  <svelte:component this={getOSIcon(submission)} size={10} class="text-gray-500" />
                  {getPlatformLabel(submission)}
                </span>
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {getTitle(submission)}
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span
                class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full
                {submission.status === 'completed'
                  ? 'bg-green-100 text-green-800'
                  : submission.status === 'failed'
                    ? 'bg-red-100 text-red-800'
                    : 'bg-yellow-100 text-yellow-800'}">
                {submission.status}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <div class="flex items-center space-x-1">
                <Clock size={14} class="text-gray-400" />
                <span>{formatDuration(submission)}</span>
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {getTotalFileSize(submission)}
            </td>
          </tr>
        {/each}
        {#if submissions.length === 0}
          <tr>
            <td colspan="5" class="px-6 py-8 text-center text-gray-500">
              No submissions yet. Submissions will appear here when users upload demonstrations.
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  {/if}
</Card>

<DownloadScriptsModal 
  open={showDownloadModal}
  submissions={submissions}
  onClose={() => (showDownloadModal = false)}
/>
