<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import EventTimestamp from '$lib/components/gym/EventTimestamp.svelte';
  import AxTreeOverlay from '$lib/components/gym/AxTreeOverlay.svelte';
  import { Copy, ExternalLink, Asterisk, Equal } from 'lucide-svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import type { Recording } from '$lib/gym';
  import { getPlatform } from '$lib/utils';
  import { getSubmissionStatus } from '$lib/api/forge';
  import type { SubmissionStatus } from '$lib/types/forge';
  import { walletAddress } from '$lib/stores/wallet';
  import { listSubmissions } from '$lib/api/forge';
  import { handleUpload, uploadQueue, saveUploadConfirmation } from '$lib/uploadManager';
  import UploadConfirmModal from '$lib/components/UploadConfirmModal.svelte';

  let platform: Awaited<ReturnType<typeof getPlatform>> = 'windows';
  let submissions: SubmissionStatus[] = [];
  let showDetails = false;

  const recordingId = new URLSearchParams(window.location.search).get('id');
  let selectedView: 'raw' | 'sft' | 'grpo' = 'raw';
  let recording: Recording | null = null;
  let processing = false;
  let checkingData = false;
  let uploading = false;
  let submission: SubmissionStatus | null = null;
  let submissionError: string | null = null;
  let showUploadConfirmModal = false;

  // Poll submission status
  let statusInterval: number | null = null;
  function pollSubmissionStatus(submissionId: string) {
    statusInterval = setInterval(async () => {
      try {
        const status = await getSubmissionStatus(submissionId);
        submission = status;
        if (status.status === 'completed' || status.status === 'failed') {
          if (statusInterval) {
            clearInterval(statusInterval);
            statusInterval = null;
          }
        }
      } catch (error) {
        console.error('Failed to get submission status:', error);
      }
    }, 5000);
  }

  // Clean up interval on unmount
  onDestroy(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
  });

  // Track upload status from the uploadQueue store
  $: {
    const uploadStatus = $uploadQueue[recordingId || ''];
    uploading = uploadStatus?.status === 'uploading' || uploadStatus?.status === 'processing';

    if (uploadStatus?.status === 'failed' && uploadStatus.error) {
      submissionError = uploadStatus.error;
    } else if (uploadStatus?.status === 'completed') {
      submissionError = null;
    }
  }

  // Function to handle upload button click
  async function handleUploadClick() {
    if (recordingId) {
      const uploadStarted = await handleUpload(recordingId, recording?.title || 'Unknown');
      if (!uploadStarted) {
        // If upload didn't start, show confirmation modal
        showUploadConfirmModal = true;
      }
    }
  }

  // Handle confirmation modal actions
  function handleConfirmUpload() {
    saveUploadConfirmation(true);
    if (recordingId) {
      handleUpload(recordingId, recording?.title || 'Unknown');
    }
  }

  function handleCancelUpload() {
    // Do nothing, just close the modal
  }

  async function checkProcessedData() {
    checkingData = true;
    try {
      // Check for SFT data
      try {
        sftHtml = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'sft.html'
        });
      } catch (error) {
        sftHtml = null;
      }

      // Check for GRPO data
      try {
        grpoHtml = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'grpo.html'
        });
      } catch (error) {
        grpoHtml = null;
      }
    } finally {
      checkingData = false;
    }
  }

  async function handleProcess() {
    try {
      processing = true;
      await invoke('process_recording', { recordingId });
      // Check for processed data after pipeline completes
      await checkProcessedData();
    } catch (error) {
      console.error('Failed to process recording:', error);
    } finally {
      processing = false;
    }
  }

  let rawEvents: Array<{ time: number; event: string; data: any }> = [];
  let currentPage = 0;
  const itemsPerPage = 100;
  $: filteredEvents = rawEvents.filter((event) => enabledEventTypes.has(event.event));
  $: pagesWithEvents = Array.from(
    { length: Math.ceil(filteredEvents.length / itemsPerPage) },
    (_, i) => {
      const pageEvents = filteredEvents.slice(i * itemsPerPage, (i + 1) * itemsPerPage);
      return pageEvents.length > 0 ? i : -1;
    }
  ).filter((i) => i !== -1);

  $: {
    // Auto-advance to next page with events when current page is empty
    const currentPageEvents = filteredEvents.slice(
      currentPage * itemsPerPage,
      (currentPage + 1) * itemsPerPage
    );
    if (currentPageEvents.length === 0 && pagesWithEvents.length > 0) {
      const nextPage = pagesWithEvents.find((p) => p > currentPage);
      if (nextPage !== undefined) {
        currentPage = nextPage;
      } else {
        currentPage = pagesWithEvents[0]; // Wrap to first page with events
      }
    }
  }
  let sftHtml: string | null = null;
  let grpoHtml: string | null = null;
  let videoSrc: string | null = null;
  let videoElement: HTMLVideoElement | null = null;
  let startTimestamp: number | null = null;
  let eventTypes: Set<string> = new Set();
  let enabledEventTypes: Set<string> = new Set();
  let visibleAxTree: number | null = null;

  async function copyEventData(event: any) {
    await writeText(JSON.stringify(event, null, 2));
  }

  onMount(async () => {
    try {
      const recordings = await invoke<Recording[]>('list_recordings');
      recording = recordings.find((r) => r.id === recordingId) || null;

      platform = await getPlatform();

      // Load video
      try {
        videoSrc = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'recording.mp4',
          asBase64: true
        });
      } catch (error) {
        console.error('Failed to load video:', error);
      }

      // Load raw events
      try {
        const text = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'input_log.jsonl'
        });
        const events = text
          .split('\n')
          .filter((line) => line.trim())
          .map((line) => {
            try {
              return JSON.parse(line);
            } catch {
              return null;
            }
          })
          .filter(
            (event): event is { time: number; event: string; data: any } =>
              event !== null && typeof event === 'object' && typeof event.time === 'number'
          );

        // Set start timestamp from first event
        if (events.length > 0) {
          startTimestamp = events[0].time;
          rawEvents = events;
          // Extract unique event types
          eventTypes = new Set(events.map((e) => e.event));
          enabledEventTypes = new Set(eventTypes);
        }
      } catch (error) {
        console.log('Raw events not available');
      }

      await checkProcessedData();

      // Load submissions if wallet connected
      if ($walletAddress) {
        submissions = await listSubmissions($walletAddress);
        submission = submissions.find((s) => s.meta?.id === recordingId) || null;
      }
    } catch (error) {
      console.error('Failed to load recording:', error);
    }
  });

  // Subscribe to wallet address changes
  walletAddress.subscribe((val) => {
    if (val !== $walletAddress && val) {
      listSubmissions(val).then((subs) => {
        submissions = subs;
        submission = submissions.find((s) => s.meta?.id === recordingId) || null;
      });
    }
  });

  function getLetterGrade(score: number): string {
    if (score >= 90) return 'A';
    if (score >= 80) return 'B';
    if (score >= 70) return 'C';
    if (score >= 60) return 'D';
    return 'F';
  }

  function truncateHash(hash: string): string {
    return `${hash.slice(0, 4)}...${hash.slice(-4)}`;
  }

  function getSolscanUrl(txHash: string): string {
    return `https://solscan.io/tx/${txHash}`;
  }

  function formatNumber(num: number): string {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function formatJson(event: { time: number; event: string; data: any }) {
    return JSON.stringify(event, null, 2);
  }
</script>

<div class="h-full max-w-7xl mx-auto">
  <div>
    {#if recording}
      <div class="flex gap-3 xl:flex-row flex-col h-[calc(100vh-8rem)]">
        <!-- Video Section -->
        <div class="w-full flex flex-col">
          <Card padding="none" className="mb-6">
            <div class="relative w-full">
              <video bind:this={videoElement} controls class="w-full h-full" src={videoSrc || ''}>
                Your browser does not support the video tag.
              </video>
              {#if visibleAxTree !== null}
                {#each filteredEvents as event}
                  {#if event.time === visibleAxTree && event.event === 'axtree'}
                    <AxTreeOverlay
                      tree={event.data.tree}
                      {videoElement}
                      resolution={{
                        width: recording.primary_monitor.width,
                        height: recording.primary_monitor.height
                      }} />
                  {/if}
                {/each}
              {/if}
            </div>
          </Card>

          {#if submission?.grade_result}
            <Card padding="lg" className="mb-6">
              <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <div class="text-xl font-semibold">Submission Result</div>
                    {#if submission?.clampedScore}
                      <div
                        class="text-lg font-medium"
                        class:text-red-500={submission.clampedScore < 50}
                        class:text-secondary-300={submission.clampedScore >= 50}>
                        ({getLetterGrade(submission.clampedScore)})
                      </div>
                    {/if}
                  </div>
                  <div class="flex items-center gap-2">
                    {#if submission?.treasuryTransfer?.txHash}
                      <a
                        href={getSolscanUrl(submission.treasuryTransfer.txHash)}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-gray-400 hover:text-secondary-300 transition-colors">
                        <Button variant="secondary" class="text-sm flex! items-center gap-1">
                          <ExternalLink class="w-3.5 h-3.5" />
                          Solscan
                        </Button>
                      </a>
                    {/if}
                    <Button
                      variant="secondary"
                      class="text-sm"
                      onclick={() => (showDetails = !showDetails)}>
                      {showDetails ? 'Hide Details' : 'Show Details'}
                    </Button>
                  </div>
                </div>

                <div class="text-gray-500 italic">{submission.grade_result.reasoning}</div>

                <div class="flex flex-wrap items-center gap-3 mt-4">
                  {#if submission?.maxReward}
                    <div class="bg-gray-100 rounded-lg px-3 py-2">
                      <div class="text-sm text-gray-500">Price per Demo</div>
                      <div class="text-secondary-300 font-medium">
                        {formatNumber(submission.maxReward)} VIRAL
                      </div>
                    </div>
                  {/if}
                  <Asterisk class="w-4 h-4 text-gray-400" />
                  {#if submission?.clampedScore}
                    <div class="bg-gray-100 rounded-lg px-3 py-2">
                      <div class="text-sm text-gray-500">Quality Score</div>
                      <div class="text-secondary-300 font-medium">{submission.clampedScore}%</div>
                    </div>
                  {/if}
                  <Equal class="w-4 h-4 text-gray-400" />
                  {#if submission?.reward}
                    <div class="bg-gray-100 rounded-lg px-3 py-2">
                      <div class="text-sm text-gray-500">Earned</div>
                      <div class="text-secondary-300 font-medium">
                        {formatNumber(submission.reward)} VIRAL
                      </div>
                    </div>
                  {/if}
                </div>

                {#if showDetails}
                  <div class="text-gray-600 max-h-[300px] overflow-y-auto pr-2 mt-4">
                    {#if submission.treasuryTransfer?.txHash}
                      <div class="flex items-center gap-2 mb-4">
                        <span class="text-gray-500">TX:</span>
                        <span class="text-gray-400 font-mono text-xs">
                          {submission.treasuryTransfer.txHash}
                        </span>
                        <button
                          class="text-gray-400 hover:text-secondary-300 transition-colors"
                          onclick={() => {
                            const txHash = submission?.treasuryTransfer?.txHash;
                            if (txHash) navigator.clipboard.writeText(txHash);
                          }}>
                          <Copy class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    {/if}
                    <div class="whitespace-pre-wrap">{submission.grade_result.summary}</div>
                  </div>
                {/if}
              </div>
            </Card>
          {:else}
            <Card padding="lg" className="mb-6">
              <div class="flex items-center gap-2">
                <div class="text-xl w-1/3 font-semibold">Submission Result</div>
                <div class="italic w-2/3 text-gray-500">
                  Get your submission grade and receive your $VIRAL by uploading this demonstration.
                </div>
              </div>
            </Card>
          {/if}

          <Card padding="lg">
            <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
              <div class="text-gray-700 min-w-0">
                <div class="font-medium truncate">{recording.title}</div>
                <div class="text-sm text-gray-500 grid grid-cols-2 gap-x-4 gap-y-1 mt-1">
                  <div>ID: {recording.id}</div>
                  <div>
                    Resolution: {recording.primary_monitor.width}x{recording.primary_monitor.height}
                  </div>
                  <div>OS: {recording.platform} {recording.version} ({recording.arch})</div>
                  <div>Locale: {recording.locale}</div>
                </div>
              </div>
              <div class="flex gap-2 shrink-0">
                <Button
                  variant="secondary"
                  onclick={() => invoke('open_recording_folder', { recordingId })}>
                  Open in {#if platform === 'windows'}
                    Explorer
                  {:else if platform === 'macos'}
                    Finder
                  {:else}
                    Files
                  {/if}
                </Button>
                {#if recording.status === 'completed'}
                  <Button
                    variant="secondary"
                    onclick={handleProcess}
                    class="flex! items-center"
                    disabled={processing || checkingData}>
                    {#if processing}
                      <div
                        class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                      </div>
                      Processing...
                    {:else if checkingData}
                      <div
                        class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                      </div>
                      Checking...
                    {:else}
                      Process
                    {/if}
                  </Button>
                  <Button
                    variant="primary"
                    onclick={handleUploadClick}
                    class="flex! items-center"
                    disabled={uploading ||
                      !$walletAddress ||
                      submission?.status === 'completed' ||
                      (submission && submission.status !== 'failed')}>
                    {#if uploading}
                      <div
                        class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                      </div>
                      Uploading...
                    {:else if submission}
                      {#if submission.status === 'completed'}
                        ✓ Uploaded
                      {:else if submission.status === 'failed'}
                        Failed
                      {:else}
                        Processing...
                      {/if}
                    {:else}
                      Upload
                    {/if}
                  </Button>
                {/if}
              </div>
            </div>

            {#if submissionError}
              <p class="text-red-500 text-sm mt-2">Upload Error: {submissionError}</p>
            {/if}
          </Card>
        </div>

        <!-- Data Section -->
        <div class="w-full xl:w-1/2 xl:max-w-md">
          <Card padding="lg" className="h-full">
            <div class="flex flex-col gap-4 mb-4">
              <div class="flex gap-2">
                <Button
                  variant={selectedView === 'raw' ? 'primary' : 'secondary'}
                  onclick={() => (selectedView = 'raw')}
                  class="flex-1">
                  Raw Events
                </Button>
                <Button
                  variant={selectedView === 'sft' ? 'primary' : 'secondary'}
                  onclick={() => (selectedView = 'sft')}
                  class="flex-1">
                  SFT Data
                </Button>
                <Button
                  variant={selectedView === 'grpo' ? 'primary' : 'secondary'}
                  onclick={() => (selectedView = 'grpo')}
                  class="flex-1">
                  GRPO Data
                </Button>
              </div>

              {#if selectedView === 'raw'}
                <div class="flex flex-wrap gap-2">
                  {#each Array.from(eventTypes) as type}
                    <Button
                      variant="secondary"
                      class={`text-sm ${!enabledEventTypes.has(type) ? 'opacity-50' : ''} p-1! px-2!`}
                      onclick={() => {
                        if (enabledEventTypes.has(type)) {
                          enabledEventTypes.delete(type);
                        } else {
                          enabledEventTypes.add(type);
                        }
                        enabledEventTypes = enabledEventTypes; // Trigger reactivity
                      }}>
                      {type}
                    </Button>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="h-full overflow-auto">
              {#if selectedView === 'raw'}
                <div class="flex flex-col">
                  {#each filteredEvents.slice(currentPage * itemsPerPage, (currentPage + 1) * itemsPerPage) as event, i}
                    <div
                      class={`flex gap-2 items-start min-w-0 p-2 rounded ${i % 2 === 0 ? 'bg-gray-100' : ''}`}>
                      <div class="flex flex-col gap-1 min-w-[50px] select-none">
                        <EventTimestamp
                          timestamp={event.time}
                          startTime={startTimestamp || 0}
                          {videoElement} />
                        {#if event.event === 'axtree'}
                          <Button
                            variant="secondary"
                            class="text-sm p-0! px-1! w-full"
                            onclick={() =>
                              (visibleAxTree = visibleAxTree === event.time ? null : event.time)}>
                            {visibleAxTree === event.time ? 'Hide Tree' : 'Show Tree'}
                          </Button>
                        {/if}
                        <Button
                          variant="secondary"
                          class="text-sm p-0! px-1! w-full"
                          onclick={() => copyEventData(event)}>
                          Copy
                        </Button>
                      </div>
                      <pre
                        class="text-sm text-gray-700 whitespace-pre-wrap break-all flex-1 overflow-hidden">{formatJson(
                          event
                        )}</pre>
                    </div>
                  {/each}

                  {#if filteredEvents.length > 0}
                    <div
                      class="flex items-center justify-between mt-4 sticky bottom-0 bg-white p-2 w-full">
                      <Button
                        variant="secondary"
                        onclick={() => {
                          const prevPage = [...pagesWithEvents]
                            .reverse()
                            .find((p) => p < currentPage);
                          if (prevPage !== undefined) {
                            currentPage = prevPage;
                          } else {
                            currentPage = pagesWithEvents[pagesWithEvents.length - 1]; // Wrap to last page
                          }
                        }}
                        disabled={!pagesWithEvents.length ||
                          (pagesWithEvents.length === 1 && pagesWithEvents[0] === currentPage)}>
                        Previous
                      </Button>

                      <span class="text-sm text-gray-600">
                        Page {currentPage + 1} of {pagesWithEvents.length}
                      </span>

                      <Button
                        variant="secondary"
                        onclick={() => {
                          const nextPage = pagesWithEvents.find((p) => p > currentPage);
                          if (nextPage !== undefined) {
                            currentPage = nextPage;
                          } else {
                            currentPage = pagesWithEvents[0]; // Wrap to first page
                          }
                        }}
                        disabled={!pagesWithEvents.length ||
                          (pagesWithEvents.length === 1 && pagesWithEvents[0] === currentPage)}>
                        Next
                      </Button>
                    </div>
                  {/if}
                </div>
              {:else if selectedView === 'sft'}
                {#if sftHtml}
                  {@html sftHtml}
                {:else}
                  <div class="text-center py-8 text-gray-500">
                    <p>SFT data not available.</p>
                    <p>Click the Process button to generate SFT data.</p>
                    {#if processing}
                      <p class="mt-4">Processing... please wait.</p>
                    {/if}
                  </div>
                {/if}
              {:else if selectedView === 'grpo'}
                {#if grpoHtml}
                  {@html grpoHtml}
                {:else}
                  <div class="text-center py-8 text-gray-500">
                    <p>GRPO data not available.</p>
                    <p>Click the Process button to generate GRPO data.</p>
                    {#if processing}
                      <p class="mt-4">Processing... please wait.</p>
                    {/if}
                  </div>
                {/if}
              {/if}
            </div>
          </Card>
        </div>
      </div>
    {:else}
      <div class="text-center py-8 text-gray-500">Loading recording details...</div>
    {/if}
  </div>
</div>

<UploadConfirmModal
  open={showUploadConfirmModal}
  onConfirm={handleConfirmUpload}
  onCancel={handleCancelUpload} />
