<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Search, Upload, Edit, Copy, ExternalLink } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Recording } from '$lib/gym';
  import { walletAddress } from '$lib/stores/wallet';
  import { getSubmissionStatus, listSubmissions } from '$lib/api/forge';
  import type { SubmissionStatus } from '$lib/api/forge';
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
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
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
</script>

<div class="h-full max-w-7xl mx-auto">
  <div class="">
    {#if dataExported !== ''}
      <div
        transition:fly={{ x: 0, y: 200 }}
        class="absolute right-10 top-10 bg-gray-700 rounded-lg p-5 z-50">
        <p class="text-emerald-500 font-semibold">Data Export Complete!</p>
        <button
          onclick={() => open(dataExported.substring(0, dataExported.lastIndexOf('/') + 1))}
          class="text-sm text-gray-300 hover:underline cursor-pointer hover:text-white">
          {dataExported}
        </button>
      </div>
    {/if}
    <button
      onclick={() => invoke('open_recording_folder', { recordingId: '' })}
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

  <div class="grid gap-4 pb-4">
    {#each filteredRecordings as recording}
      <Card padding="lg" className="border-2">
        <div class="flex flex-col sm:flex-row gap-4">
          <div class="grow">
            <h3 class="text-xl font-title mb-2">{recording.title}</h3>
            <p class="text-gray-700 mb-3">{recording.description || 'No description'}</p>
            <div class="flex flex-wrap gap-4 text-sm text-gray-600">
              <span>{Math.round(recording.duration_seconds / 60)} minutes</span>
              <span>Status: {recording.status}</span>
              <span>Recorded: {new Date(recording.timestamp).toLocaleString()}</span>
            </div>
            <div class="text-sm text-gray-500 pt-1">
              <span class="uppercase font-semibold">Rewarded:</span>

              {#if getRewardDisplay(recording)}
                <span class="text-secondary-300">{getRewardDisplay(recording)}</span>
                {#if recording.submission?.treasuryTransfer?.txHash}
                  <div class="mt-1 flex items-center gap-2">
                    <span class="text-gray-500">TX:</span>
                    <span class="text-gray-400 font-mono text-xs">
                      {truncateHash(recording.submission?.treasuryTransfer?.txHash || '')}
                    </span>
                    <div class="flex items-center gap-2">
                      <button 
                        class="text-gray-400 hover:text-secondary-300 transition-colors"
                        onclick={() => {
                          const txHash = recording.submission?.treasuryTransfer?.txHash;
                          if (txHash) navigator.clipboard.writeText(txHash);
                        }}>
                        <Copy class="w-3.5 h-3.5" />
                      </button>
                      <a
                        href={getSolscanUrl(recording.submission?.treasuryTransfer?.txHash || '')}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-gray-400 hover:text-secondary-300 transition-colors">
                        <ExternalLink class="w-3.5 h-3.5" />
                      </a>
                    </div>
                  </div>
                {/if}
              {:else}
                <span class="text-gray-500 italic">
                  Upload your recording to get a score and receive your $VIRAL.
                </span>
              {/if}
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <a href="/app/gym/history/recording?id={recording.id}" class="block">
              <Button variant="secondary" class="h-8 text-sm flex! items-center w-full">
                <Edit class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                <span>Details</span>
              </Button>
            </a>
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
                class="h-8 text-sm flex! items-center"
                disabled={$uploadQueue[recording.id]?.status === 'uploading' || !$walletAddress}>
                {#if $uploadQueue[recording.id]?.status === 'uploading'}
                  <div
                    class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                  </div>
                  <span>Uploading...</span>
                {:else}
                  <Upload class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                  <span>Upload</span>
                {/if}
              </Button>
            {/if}
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
