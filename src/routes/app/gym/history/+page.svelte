<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import PopupMenu from '$lib/components/PopupMenu.svelte';
  import MenuItem from '$lib/components/MenuItem.svelte';
  import {
    Search,
    Upload,
    AlertTriangle,
    MoreVertical,
    Clock,
    Calendar,
    Folder,
    Trash2,
    Eye,
    Download,
    Info
  } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Recording } from '$lib/gym';
  import { walletAddress } from '$lib/stores/wallet';
  import { getSubmissionStatus, listSubmissions } from '$lib/api/forge';
  import type { SubmissionStatus } from '$lib/types/forge';
  import UploadConfirmModal from '$lib/components/UploadConfirmModal.svelte';
  import { fly } from 'svelte/transition';
  import { open } from '@tauri-apps/plugin-shell';
  import { uploadManager } from '$lib/stores/misc';
  import type { UploadQueueItem } from '$lib/uploadManager';
  import { ask } from '@tauri-apps/plugin-dialog';

  let searchQuery = $state('');
  let exporting = $state(false);
  let exportingZip = $state(false);
  let sortOrder: 'newest' | 'oldest' = $state('newest');
  let recordings: Recording[] = $state([]);
  let submissions: SubmissionStatus[] = $state([]);
  let showUploadConfirmModal = $state(false);
  let dataExported = $state('');
  let statusIntervals: { [key: string]: number } = {};
  let uploadConfirmRecording: Recording = $state({} as Recording);

  onDestroy(() => {
    // Clear all intervals
    Object.values(statusIntervals).forEach((interval) => {
      clearInterval(interval);
    });
  });

  function formatNumber(num: number): string {
    if (num === undefined || num === null) return '0.00';
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function formatDuration(seconds: number): string {
    if (!seconds) return '0:00';

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

  onMount(async () => {
    try {
      if ($walletAddress) {
        listSubmissions();
      }
      recordings = await invoke('list_recordings');

      $uploadManager.on('statusChange', '*', async () => {
        if ($walletAddress) await listSubmissions();
        recordings = await invoke('list_recordings');
      });
    } catch (error) {
      console.error('Failed to fetch recordings:', error);
    }
  });

  // Subscribe to wallet address changes
  walletAddress.subscribe((val) => {
    if (val !== $walletAddress && val) {
      listSubmissions();
    }
  });

  const mergedRecordings = $derived([
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
  ]);

  const filteredRecordings = $derived(
    mergedRecordings
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
      })
  );

  async function handleExport() {
    exporting = true;
    const res = (await invoke('export_recordings')) as string;
    exporting = false;
    dataExported = res;
    setTimeout(() => (dataExported = ''), 5000);
  }

  function handleImageError(event: Event) {
    const img = event.target as HTMLImageElement;
    img.src = 'https://placehold.co/40x40/gray/white?text=App';
  }

  function getMaxReward(
    recording: Recording & { submission?: SubmissionStatus; quest?: any }
  ): number {
    // if (
    //   (recording.meta?.quest?.reward?.max_reward ||
    //     recording.submission?.meta?.quest?.reward?.max_reward ||
    //     recording.quest?.maxReward ||
    //     0) == 0
    // )
    //   console.log(recording);
    return (
      recording.meta?.quest?.reward?.max_reward ||
      recording.submission?.meta?.quest?.reward?.max_reward ||
      recording.quest?.reward?.max_reward ||
      0
    );
  }

  const uploadRecording = async (recording: Recording) => {
    const uploadStarted = await $uploadManager.handleUpload(recording.id, recording.title);
    if (!uploadStarted) {
      uploadConfirmRecording = recording;
      showUploadConfirmModal = true;
    }
  };

  function isUploaded(recording: Recording & { submission?: SubmissionStatus }): boolean {
    return !!recording.submission;
  }

  const uploadQueue = $uploadManager.queue;
  const isUploading = (item: UploadQueueItem) => {
    if (!item) return false;
    return item.status === 'processing' || item.status === 'uploading' || item.status === 'zipping';
  };

  // Popup menu state
  let showMenu = $state(false);
  let menuPosition = $state({ x: 0, y: 0 });
  let activeRecordingId: string | null = null;

  // Handle menu button click
  function handleMenuClick(event: MouseEvent, recordingId: string) {
    event.preventDefault();
    event.stopPropagation();

    // If the menu is already open and the same recording is clicked, close the menu
    if (showMenu && activeRecordingId === recordingId) {
      showMenu = false;
      return;
    }

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    menuPosition = {
      x: rect.left - 200,
      y: rect.bottom + window.scrollY
    };

    activeRecordingId = recordingId;
    showMenu = true;
  }

  // Export recording as zip
  async function exportRecordingZip(recordingId: string) {
    if (!recordingId) return;

    try {
      exportingZip = true;

      // Create the zip file using the Rust backend
      const zipBytes = await invoke<number[]>('create_recording_zip', { recordingId });

      // Convert to Blob and create a download link
      const zipBlob = new Blob([Uint8Array.from(zipBytes)], { type: 'application/zip' });
      const url = URL.createObjectURL(zipBlob);

      // Create a timestamp for the filename
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `export_zip_${timestamp}.zip`;

      // Create a download link and click it
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();

      // Clean up
      setTimeout(() => {
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      }, 100);
    } catch (error) {
      console.error('Failed to export zip:', error);
      alert(`Error exporting zip: ${error}`);
    } finally {
      exportingZip = false;
    }
  }

  // Delete recording function
  async function deleteRecording(recordingId: string) {
    const res = await ask(
      'Are you sure you want to delete this recording? This action cannot be undone.',
      {
        title: 'Delete Recording',
        kind: 'warning'
      }
    );
    if (res) {
      try {
        await invoke('delete_recording', { recordingId });
        // Refresh recordings list
        if ($walletAddress) await listSubmissions();
        recordings = await invoke('list_recordings');
        //todo: figure out why this doesn't remove the existing recording from the list
      } catch (error) {
        console.error('Failed to delete recording:', error);
        alert(`Error. Recording deletion failed.\n${error}`);
      }
    }
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
              onerror={handleImageError} />
          {:else}
            <div
              class="w-8 h-8 bg-gray-200 rounded-md flex items-center justify-center text-gray-500">
              <span class="text-xs">App</span>
            </div>
          {/if}

          <!-- Title and Metadata -->
          <div class="flex-grow min-w-0">
            <a href="/app/gym/history/recording?id={recording.id}" class="hover:underline">
              <h3 class="text-base font-title truncate" title={recording.title}>
                {recording.title}
              </h3>
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

          {#if recording.submission?.status === 'processing'}
            <div class="text-center px-2">
              <div class="text-yellow-600 flex items-center gap-1">
                <Info class="w-4 h-4" />
                <span class="text-sm font-semibold">Processing</span>
              </div>
            </div>
          {/if}
          <!-- Rating (if claimed) -->
          {#if recording.submission?.clampedScore !== undefined}
            <div class="text-center px-2">
              <div class="text-lg font-bold text-secondary-300">
                {recording.submission.clampedScore}%
              </div>
              <div class="text-xs text-gray-500">Rating</div>
            </div>
          {:else if recording.submission?.grade_result.score}
            <div class="text-center px-2">
              <div class="text-lg font-bold text-secondary-300">
                {recording.submission.grade_result.score}%
              </div>
              <div class="text-xs text-gray-500">Rating</div>
            </div>
          {:else if recording.submission}
            <div class="text-center px-2">
              <div class="text-lg font-bold text-secondary-300">0%</div>
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
          {:else if recording.submission}
            <div class="text-right min-w-[120px]">
              <div class="text-sm font-semibold text-secondary-300">0 VIRAL</div>
            </div>
          {/if}

          <!-- Action Buttons -->
          <div class="flex gap-2 items-center">
            {#if recording.status === 'completed' && !recording.submission && $uploadQueue[recording.id]?.status !== 'completed'}
              {#if $uploadQueue[recording.id]?.status === 'failed'}
                <div class="text-center px-2">
                  <div class="text-red-500 flex items-center gap-1">
                    <AlertTriangle class="w-4 h-4" />
                    <span class="text-sm font-semibold">Upload Failed</span>
                  </div>
                </div>
              {/if}
              <Button
                onclick={() => uploadRecording(recording)}
                class="h-8 flex! items-center gap-1.5 px-2! text-sm font-semibold"
                title="Upload Recording to earn VIRAL tokens"
                disabled={isUploading($uploadQueue[recording.id]) ||
                  $uploadQueue[recording.id]?.status === 'queued' ||
                  !$walletAddress}>
                {#if isUploading($uploadQueue[recording.id])}
                  <div
                    class="w-3.5 h-3.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                  </div>
                  <span>Uploading...</span>
                {:else if $uploadQueue[recording.id]?.status === 'queued'}
                  Queued
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

            <!-- Options Button -->
            <Button
              variant="secondary"
              class="h-8 w-8 p-0! flex! items-center justify-center bg-transparent shadow-transparent border-transparent"
              title="Recording Options"
              onclick={(e: MouseEvent) => handleMenuClick(e, recording.id)}>
              <MoreVertical class="w-4 h-4 shrink-0" />
            </Button>
          </div>
        </div>
      </Card>
    {/each}
  </div>

  <UploadConfirmModal
    bind:open={showUploadConfirmModal}
    uploader={async () => {
      uploadRecording(uploadConfirmRecording);
    }} />

  <!-- Popup Menu -->
  <PopupMenu bind:open={showMenu} position={menuPosition} on:close={() => (showMenu = false)}>
    <MenuItem
      icon={Folder}
      on:click={() => {
        if (activeRecordingId) {
          invoke('open_recording_folder', { recordingId: activeRecordingId });
        }
        showMenu = false;
      }}>
      Open Folder
    </MenuItem>
    <MenuItem
      icon={Download}
      on:click={() => {
        if (activeRecordingId) {
          exportRecordingZip(activeRecordingId);
        }
        showMenu = false;
      }}>
      {exportingZip ? 'Exporting...' : 'Export Zip'}
    </MenuItem>
    {#if !submissions.find((s) => s.meta.id === activeRecordingId)}
      <MenuItem
        icon={Trash2}
        danger
        on:click={() => {
          if (activeRecordingId) {
            deleteRecording(activeRecordingId);
          }
          showMenu = false;
        }}>
        Delete
      </MenuItem>
    {/if}
  </PopupMenu>
</div>
