<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Search, Upload, Edit, Play } from 'lucide-svelte';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Recording } from '$lib/gym';
  import { walletAddress } from '$lib/stores/wallet';
  import { getSubmissionStatus, listSubmissions, uploadRecording } from '$lib/api/forge';
  import type { SubmissionStatus } from '$lib/api/forge';

  let searchQuery = '';
  let sortOrder: 'newest' | 'oldest' = 'newest';
  let recordings: Recording[] = [];
  let submissions: SubmissionStatus[] = [];
  let processing: string | null = null;
  let uploading: string | null = null;
  let submissionError: { [key: string]: string } = {};
  let statusIntervals: { [key: string]: number } = {};

  onDestroy(() => {
    // Clear all intervals
    Object.values(statusIntervals).forEach(interval => {
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
        submissions = submissions.map(s => 
          s.meta?.id === recordingId ? status : s
        );
        if (status.status === 'completed' || status.status === 'failed') {
          clearInterval(statusIntervals[recordingId]);
          delete statusIntervals[recordingId];
        }
      } catch (error) {
        console.error('Failed to get submission status:', error);
      }
    }, 5000);
  }

  async function handleUpload(recordingId: string) {
    if (!$walletAddress) {
      submissionError[recordingId] = 'Please connect your wallet first';
      return;
    }

    try {
      uploading = recordingId;
      submissionError[recordingId] = '';
      // Get zip file as bytes
      const zipBytes = await invoke<number[]>('create_recording_zip', { recordingId });
      // Convert to Blob
      const zipBlob = new Blob([Uint8Array.from(zipBytes)], { type: 'application/zip' });
      // Upload to server
      const { submissionId } = await uploadRecording(zipBlob, $walletAddress);
      // Start polling status
      pollSubmissionStatus(recordingId, submissionId);
    } catch (error) {
      console.error('Failed to upload recording:', error);
      submissionError[recordingId] = error instanceof Error ? error.message : 'Failed to upload recording';
    } finally {
      uploading = null;
    }
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
    ...recordings.map(recording => {
      const submission = submissions.find(s => s.meta?.id === recording.id);
      return {
        ...recording,
        submission
      };
    }),
    // Then include submissions without local recordings
    ...submissions
      .filter(s => !recordings.some(r => r.id === s.meta?.id))
      .map(s => ({
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


  async function handleProcess(recordingId: string) {
    try {
      processing = recordingId;
      await invoke('process_recording', { recordingId });
    } catch (error) {
      console.error('Failed to process recording:', error);
    } finally {
      processing = null;
    }
  }

  function getRewardDisplay(recording: Recording & { submission?: SubmissionStatus }) {
    console.log(recording.submission)
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
              <span class="text-secondary-300">{getRewardDisplay(recording)}</span>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <a href="/app/gym/history/{recording.id}" class="block">
              <Button variant="secondary" class="h-8 text-sm flex! items-center w-full">
                <Edit class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                <span>Details</span>
              </Button>
            </a>
            {#if recording.status === 'completed' && !recording.submission}
              <Button
                onclick={() => handleUpload(recording.id)}
                class="h-8 text-sm flex! items-center"
                disabled={uploading === recording.id || !$walletAddress}>
                {#if uploading === recording.id}
                  <div class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin" />
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
</div>
