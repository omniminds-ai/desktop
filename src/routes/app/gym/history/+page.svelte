<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Search, Upload, MonitorPlay, Copy, ExternalLink, AlertTriangle, MoreVertical, Clock, Calendar } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Recording } from '$lib/gym';
  import { walletAddress } from '$lib/stores/wallet';
  import { getSubmissionStatus, listSubmissions } from '$lib/api/forge';
  import type { SubmissionStatus } from '$lib/types/forge';
  import { handleUpload, uploadQueue, saveUploadConfirmation } from '$lib/uploadManager';
  import UploadConfirmModal from '$lib/components/UploadConfirmModal.svelte';
  import { fly } from 'svelte/transition';
  import { open } from '@tauri-apps/plugin-shell';

  let searchQuery = '';
  let exporting = false;
  let sortOrder: 'newest' | 'oldest' = 'newest';
  let recordings: Recording[] = [];
  let submissions: SubmissionStatus[] = [];
  let showUploadConfirmModal = false;
  let pendingUploadId = '';
  let pendingUploadTitle = '';
  let dataExported = '';
  let statusIntervals: { [key: string]: number } = {};

  onDestroy(() => {
    // Clear all intervals
    Object.values(statusIntervals).forEach((interval) => {
      clearInterval(interval);
    });
  });

  function pollSubmissionStatus(recordingId: string, submissionId: string) {
    if (statusIntervals[recordingId]) {
      clearInterval(statusIntervals[recordingId]);
    }

    statusIntervals[recordingId] = setInterval(async () => {
      try {
        const status = await getSubmissionStatus(submissionId);
        submissions = submissions.map((s) => (s.meta?.id === recordingId ? status : s));
        if (status.status === 'completed' || status.status === 'failed') {
          clearInterval(statusIntervals[recordingId]);
          delete statusIntervals[recordingId];
        }
      } catch (error) {
        console.error('Failed to get submission status:', error);
      }
    }, 5000);
  }

  function formatNumber(num: number): string {
    if (num === undefined || num === null) return "0.00";
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function formatDuration(seconds: number): string {
    if (!seconds) return "0:00";
    
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    
    // Format as m:ss with leading zeros for seconds when needed
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString();
  }

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  async function loadSubmissions(address: string) {
    try {
      submissions = await listSubmissions(address);
    } catch (error) {
      console.error('Failed to fetch submissions:', error);
    }
  }

  onMount(async () => {
    try {
      recordings = await invoke('list_recordings');
      if ($walletAddress) {
        loadSubmissions($walletAddress);
      }
    } catch (error) {
      console.error('Failed to fetch recordings:', error);
    }
  });

  // Subscribe to wallet address changes
  $: if ($walletAddress) {
    loadSubmissions($walletAddress);
  }

  $: mergedRecordings = [
    // First include all local recordings with their submissions
    ...recordings.map((recording) => {
      const submission = submissions.find((s) => s.meta?.id === recording.id);
      return {
        ...recording,
        submission
      };
    }),
    // Then include submissions without local recordings
    ...submissions
      .filter((s) => !recordings.some((r) => r.id === s.meta?.id))
      .map((s) => ({
        id: s.meta.id,
        timestamp: s.meta.timestamp,
        duration_seconds: s.meta.duration_seconds,
        status: s.meta.status,
        reason: s.meta.reason,
        title: s.meta.title,
        description: s.meta.description,
        platform: s.meta.platform,
        arch: s.meta.arch,
        version: s.meta.version,
        locale: s.meta.locale,
        primary_monitor: s.meta.primary_monitor,
        meta: {
          quest: s.meta.quest
        },
        submission: s
      }))
  ];

  $: filteredRecordings = mergedRecordings
    .filter(
      (recording) =>
        recording.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (recording.description &&
          recording.description.toLowerCase().includes(searchQuery.toLowerCase()))
    )
    .sort((a, b) => {
      if (sortOrder === 'newest') {
        return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
      } else {
        return new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime();
      }
    });

  async function handleExport() {
    exporting = true;
    const res = (await invoke('export_recordings')) as string;
    exporting = false;
    dataExported = res;
    setTimeout(() => (dataExported = ''), 5000);
  }

  function truncateHash(hash: string): string {
    return `${hash.slice(0, 4)}...${hash.slice(-4)}`;
  }

  function getSolscanUrl(txHash: string): string {
    return `https://solscan.io/tx/${txHash}`;
  }

  function getRewardDisplay(recording: Recording & { submission?: SubmissionStatus }) {
    if (recording.submission?.reward && recording.submission.maxReward) {
      return `${formatNumber(recording.submission.reward)} VIRAL (${recording.submission.clampedScore}% of ${formatNumber(recording.submission.maxReward)})`;
    } else if (recording.meta?.quest?.reward?.max_reward) {
      return `${formatNumber(recording.meta.quest.reward.max_reward)} VIRAL (unclaimed)`;
    }
    return null;
  }

  function handleImageError(event: Event) {
    const img = event.target as HTMLImageElement;
    img.src = 'https://placehold.co/40x40/gray/white?text=App';
  }

  function getMaxReward(recording: Recording & { submission?: SubmissionStatus, quest?: any }): number {
    if((recording.meta?.quest?.reward?.max_reward || 
           recording.submission?.meta?.quest?.reward?.max_reward || 
           recording.quest?.maxReward || 
           0) == 0) console.log(recording)
    return recording.meta?.quest?.reward?.max_reward || 
           recording.submission?.meta?.quest?.reward?.max_reward || 
           recording.quest?.reward?.max_reward || 
           0;
  }

  function isUploaded(recording: Recording & { submission?: SubmissionStatus }): boolean {
    return !!recording.submission;
  }
</script>

<div class="h-full max-w-7xl mx-auto">
  <div class="">
    {#if dataExported !== ''}
      <div
        transition:fly={{ x: 0, y: 200 }}
        class="absolute right-10 top-10 bg-gray-700 rounded-lg p-5 z-50">
        <p class="text-emerald-500 font-semibold">Data Export Complete!</p>
        <button
          on:click={() => open(dataExported.substring(0, dataExported.lastIndexOf('/') + 1))}
          class="text-sm text-gray-300 hover:underline cursor-pointer hover:text-white">
          {dataExported}
        </button>
      </div>
    {/if}
    <button
      on:click={() => invoke('open_recording_folder', { recordingId: '' })}
      class="text-secondary-300 text-sm cursor-pointer mb-2 -mt-2 hover:underline">
      Open Recordings Folder
    </button>

    <div class="flex flex-col sm:flex-row gap-4 mb-6">
      <div class="relative flex-grow">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search recordings..."
          class="w-full pl-10 pr-4 py-2 bg-white border-2 border-gray-200 rounded-lg focus:outline-none focus:border-secondary-300 text-gray-800" />
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
      </div>

      <select
        bind:value={sortOrder}
        class="px-4 py-2 bg-white border-2 border-gray-200 rounded-lg focus:outline-none focus:border-secondary-300 text-gray-800">
        <option value="newest">Newest First</option>
        <option value="oldest">Oldest First</option>
      </select>
      <Button
        behavior="none"
        variant="primary"
        disabled={exporting}
        onclick={handleExport}
        class="px-4! flex! gap-2 items-center justify-items-center"
        title="Export all your demonstration data.">
        {#if !exporting}
          Export Data
        {:else}
          <div
            class="h-5 w-5 rounded-full border-2 border-white border-t-transparent! animate-spin">
          </div>
          Exporting...
        {/if}
      </Button>
    </div>
  </div>

  <div class="grid gap-2 pb-4">
    {#each filteredRecordings as recording}
      <Card 
        padding="sm" 
        className={`hover:border-secondary-300 transition-colors ${isUploaded(recording) ? 'opacity-50 hover:opacity-75 bg-gray-100 border border-gray-300' : 'font-semibold bg-white shadow-md border-l-4 border-secondary-300'}`}>
        <div class="flex items-center gap-3">
          <!-- Icon -->
          {#if recording.meta?.quest?.icon_url || recording.submission?.meta?.quest?.icon_url}
            <img 
              src={recording.meta?.quest?.icon_url || recording.submission?.meta?.quest?.icon_url} 
              alt="App icon" 
              class="w-8 h-8 rounded-md object-contain"
              on:error={handleImageError}
            />
          {:else}
            <div class="w-8 h-8 bg-gray-200 rounded-md flex items-center justify-center text-gray-500">
              <span class="text-xs">App</span>
            </div>
          {/if}

          <!-- Title and Metadata -->
          <div class="flex-grow min-w-0">
            <a href="/app/gym/history/recording?id={recording.id}" class="hover:underline">
              <h3 class="text-base font-title truncate" title={recording.title}>{recording.title}</h3>
            </a>
            <div class="flex items-center gap-3 mt-1">
              <div class="flex items-center text-xs text-gray-500">
                <Clock class="w-3 h-3 mr-1 opacity-75" />
                <span>{formatDuration(recording.duration_seconds)}</span>
              </div>
              <div class="flex items-center text-xs text-gray-500">
                <Calendar class="w-3 h-3 mr-1 opacity-75" />
                <span>{formatDate(recording.timestamp)} {formatTime(recording.timestamp)}</span>
              </div>
            </div>
          </div>

          <!-- Failed Status -->
          {#if recording.submission?.status === 'failed'}
            <div class="text-center px-2">
              <div class="text-red-500 flex items-center gap-1">
                <AlertTriangle class="w-4 h-4" />
                <span class="text-sm font-semibold">Failed</span>
              </div>
            </div>
          {/if}

          <!-- Rating (if claimed) -->
          {#if recording.submission?.clampedScore !== undefined}
            <div class="text-center px-2">
              <div class="text-lg font-bold text-secondary-300">{recording.submission.clampedScore}%</div>
              <div class="text-xs text-gray-500">Rating</div>
            </div>
          {/if}

          <!-- Reward -->
          {#if recording.submission?.reward}
          <div class="text-right min-w-[120px]">
            <div class="text-sm font-semibold text-secondary-300">
              {formatNumber(recording.submission.reward)} VIRAL
            </div>
          </div>
          {/if}

          <!-- Action Buttons -->
          <div class="flex gap-2 items-center">
            {#if recording.status === 'completed' && !recording.submission}
              <Button
                onclick={async () => {
                  const uploadStarted = await handleUpload(recording.id, recording.title);
                  if (!uploadStarted) {
                    pendingUploadId = recording.id;
                    pendingUploadTitle = recording.title;
                    showUploadConfirmModal = true;
                  }
                }}
                class="h-8 flex! items-center gap-1.5 px-2! text-sm font-semibold"
                title="Upload Recording to earn VIRAL tokens"
                disabled={$uploadQueue[recording.id]?.status === 'uploading' || !$walletAddress}>
                {#if $uploadQueue[recording.id]?.status === 'uploading'}
                  <div
                    class="w-3.5 h-3.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                  </div>
                  <span>Uploading...</span>
                {:else}
                  <Upload class="w-3.5 h-3.5 shrink-0" />
                  <span>
                    {#if getMaxReward(recording) > 0}
                      Upload for up to {formatNumber(getMaxReward(recording))} VIRAL
                    {:else}
                      Upload Recording
                    {/if}
                  </span>
                {/if}
              </Button>
            {/if}
            
            <!-- Play/Details Button -->
            <a href="/app/gym/history/recording?id={recording.id}" class="block">
              <Button variant="secondary" class="h-8 w-8 p-0! flex! items-center justify-center bg-transparent shadow-transparent border-transparent" title="View Recording">
                <MoreVertical class="w-4 h-4 shrink-0" />
              </Button>
            </a>
          </div>
        </div>
      </Card>
    {/each}
  </div>

  <UploadConfirmModal
    open={showUploadConfirmModal}
    onConfirm={() => {
      saveUploadConfirmation(true);
      handleUpload(pendingUploadId, pendingUploadTitle);
      showUploadConfirmModal = false;
    }}
    onCancel={() => {
      showUploadConfirmModal = false;
    }} />
</div>
