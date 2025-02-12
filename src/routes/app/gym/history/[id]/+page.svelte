<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { page } from '$app/stores';
  import type { Recording } from '$lib/gym';

  const recordingId = $page.params.id;
  let selectedView: 'raw' | 'sft' | 'grpo' = 'raw';
  let recording: Recording | null = null;
  let rawEvents: string[] = [];
  let sftHtml: string | null = null;
  let grpoHtml: string | null = null;
  let videoSrc: string | null = null;

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
        rawEvents = text.split('\n').filter((line) => line.trim());
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

  function formatJson(jsonStr: string) {
    try {
      const obj = JSON.parse(jsonStr);
      return JSON.stringify(obj, null, 2);
    } catch {
      return jsonStr;
    }
  }
</script>

<div class="h-full">
  <div class="p-4">
    <GymHeader title="Recording Details" />

    {#if recording}
      <div class="flex gap-6 xl:flex-row flex-col">
        <!-- Video Section -->
        <div class="w-full xl:w-1/2">
          <Card padding="lg" className="mb-6">
            <div class="">
              <video controls class="w-full h-full" src={videoSrc || ''}>
                Your browser does not support the video tag.
              </video>
            </div>
          </Card>
        </div>

        <!-- Data Section -->
        <div class="w-full xl:w-1/2">
          <Card padding="lg">
            <div class="flex gap-2 mb-4">
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

            <div class="h-[600px] overflow-auto">
              {#if selectedView === 'raw'}
                <pre class="text-sm text-gray-700 whitespace-pre-wrap">{rawEvents
                    .map(formatJson)
                    .join('\n')}</pre>
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
