<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import EventTimestamp from '$lib/components/gym/EventTimestamp.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/state';
  import type { Recording } from '$lib/gym';

  const recordingId = page.params.id;
  let selectedView: 'raw' | 'sft' | 'grpo' = 'raw';
  let recording: Recording | null = null;
  let rawEvents: Array<{ time: number; event: string; data: any }> = [];
  let currentPage = 0;
  const itemsPerPage = 100;
  $: filteredEvents = rawEvents.filter(event => enabledEventTypes.has(event.event));
  $: pagesWithEvents = Array.from(
    { length: Math.ceil(filteredEvents.length / itemsPerPage) },
    (_, i) => {
      const pageEvents = filteredEvents.slice(i * itemsPerPage, (i + 1) * itemsPerPage);
      return pageEvents.length > 0 ? i : -1;
    }
  ).filter(i => i !== -1);

  $: {
    // Auto-advance to next page with events when current page is empty
    const currentPageEvents = filteredEvents.slice(currentPage * itemsPerPage, (currentPage + 1) * itemsPerPage);
    if (currentPageEvents.length === 0 && pagesWithEvents.length > 0) {
      const nextPage = pagesWithEvents.find(p => p > currentPage);
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

  onMount(async () => {
    try {
      const recordings = await invoke<Recording[]>('list_recordings');
      recording = recordings.find((r) => r.id === recordingId) || null;

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
        const events = text.split('\n')
          .filter((line) => line.trim())
          .map(line => {
            try {
              return JSON.parse(line);
            } catch {
              return null;
            }
          })
          .filter((event): event is { time: number; event: string; data: any } => 
            event !== null && typeof event === 'object' && typeof event.time === 'number'
          );
        
        // Set start timestamp from first event
        if (events.length > 0) {
          startTimestamp = events[0].time;
          rawEvents = events;
          // Extract unique event types
          eventTypes = new Set(events.map(e => e.event));
          enabledEventTypes = new Set(eventTypes);
        }
      } catch (error) {
        console.log('Raw events not available');
      }

      // Check for SFT data
      try {
        sftHtml = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'sft.html'
        });
      } catch (error) {
        console.log('SFT data not available yet');
      }

      // Check for GRPO data
      try {
        grpoHtml = await invoke<string>('get_recording_file', {
          recordingId,
          filename: 'grpo.html'
        });
      } catch (error) {
        console.log('GRPO data not available yet');
      }
    } catch (error) {
      console.error('Failed to load recording:', error);
    }
  });

  function formatJson(event: { time: number; event: string; data: any }) {
    return JSON.stringify(event, null, 2);
  }
</script>

<div class="h-full">
  <div class="p-4">
    <GymHeader title="Recording Details" />

    {#if recording}
      <div class="flex gap-6 xl:flex-row flex-col">
        <!-- Video Section -->
        <div class="w-full xl:w-1/2">
          <Card padding="none" className="mb-6">
            <div class="">
              <video bind:this={videoElement} controls class="w-full h-full" src={videoSrc || ''}>
                Your browser does not support the video tag.
              </video>
            </div>
          </Card>

          <Card padding="lg">
            <div class="flex items-center justify-between gap-4">
              <div class="text-gray-700 min-w-0">
                <div class="font-medium truncate">{recording.title}</div>
                <div class="text-sm text-gray-500">Recording ID: {recording.id}</div>
              </div>
              <Button
                variant="secondary"
                class="shrink-0"
                onclick={() => invoke('open_recording_folder', { recordingId })}>
                Open in Explorer
              </Button>
            </div>
          </Card>
        </div>

        <!-- Data Section -->
        <div class="w-full xl:w-1/2">
          <Card padding="lg">
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
                class="flex-1"
                disabled={!sftHtml}>
                SFT Data
              </Button>
              <Button
                variant={selectedView === 'grpo' ? 'primary' : 'secondary'}
                onclick={() => (selectedView = 'grpo')}
                class="flex-1"
                disabled={!grpoHtml}>
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
                      }}
                      >
                      {type}
                    </Button>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="h-[500px] overflow-auto">
              {#if selectedView === 'raw'}
                <div class="flex flex-col">
                  {#each filteredEvents.slice(currentPage * itemsPerPage, (currentPage + 1) * itemsPerPage) as event, i}
                    <div class={`flex gap-2 items-start min-w-0 p-2 rounded ${i % 2 === 0 ? 'bg-gray-100' : ''}`}>
                      <EventTimestamp 
                        timestamp={event.time} 
                        startTime={startTimestamp || 0}
                        {videoElement} 
                      />
                      <pre class="text-sm text-gray-700 whitespace-pre-wrap break-all flex-1 overflow-hidden">{formatJson(event)}</pre>
                    </div>
                  {/each}
                  
                  {#if filteredEvents.length > 0}
                    <div class="flex items-center justify-between mt-4 sticky bottom-0 bg-white p-2 w-full">
                      <Button
                        variant="secondary"
                        onclick={() => {
                          const prevPage = [...pagesWithEvents].reverse().find(p => p < currentPage);
                          if (prevPage !== undefined) {
                            currentPage = prevPage;
                          } else {
                            currentPage = pagesWithEvents[pagesWithEvents.length - 1]; // Wrap to last page
                          }
                        }}
                        disabled={!pagesWithEvents.length || (pagesWithEvents.length === 1 && pagesWithEvents[0] === currentPage)}>
                        Previous
                      </Button>
                      
                      <span class="text-sm text-gray-600">
                        Page {currentPage + 1} of {pagesWithEvents.length}
                      </span>
                      
                      <Button
                        variant="secondary" 
                        onclick={() => {
                          const nextPage = pagesWithEvents.find(p => p > currentPage);
                          if (nextPage !== undefined) {
                            currentPage = nextPage;
                          } else {
                            currentPage = pagesWithEvents[0]; // Wrap to first page
                          }
                        }}
                        disabled={!pagesWithEvents.length || (pagesWithEvents.length === 1 && pagesWithEvents[0] === currentPage)}>
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
                    <p>SFT data is still processing.</p>
                    <p>Please check back soon!</p>
                  </div>
                {/if}
              {:else if selectedView === 'grpo'}
                {#if grpoHtml}
                  {@html grpoHtml}
                {:else}
                  <div class="text-center py-8 text-gray-500">
                    <p>GRPO data is still processing.</p>
                    <p>Please check back soon!</p>
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
