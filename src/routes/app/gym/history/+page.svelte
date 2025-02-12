<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Search, Upload, Edit, Download, Play } from 'lucide-svelte';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { Recording } from '$lib/gym';

  let searchQuery = '';
  let sortOrder: 'newest' | 'oldest' = 'newest';
  let recordings: Recording[] = [];
  let processing: string | null = null;

  onMount(async () => {
    try {
      recordings = await invoke('list_recordings');
    } catch (error) {
      console.error('Failed to fetch recordings:', error);
    }
  });

  $: filteredRecordings = recordings
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

  function handleUpload(recordingId: string) {
    console.log('Uploading recording:', recordingId);
    // Implement upload logic here
  }
  function handleProcess(recordingId: string) {
    console.log('Processing recording:', recordingId);
    // Implement process logic here
  }
</script>

<div class="h-full">
  <div class="p-4">
    <GymHeader title="Recording History" />

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

  <div class="grid gap-4 px-4 pb-4">
    {#each filteredRecordings as recording}
      <Card padding="lg" className="border-2">
        <div class="flex flex-col sm:flex-row justify-between gap-4">
          <div>
            <h3 class="text-xl font-title mb-2">{recording.title}</h3>
            <p class="text-gray-700 mb-3">{recording.description || 'No description'}</p>
            <div class="flex flex-wrap gap-4 text-sm text-gray-600">
              <span>{Math.round(recording.duration_seconds / 60)} minutes</span>
              <span>Status: {recording.status}</span>
              <span>Recorded: {new Date(recording.timestamp).toLocaleString()}</span>
            </div>
          </div>

          <div class="flex flex-col gap-2 w-32">
            <a href="/app/gym/history/{recording.id}" class="block">
              <Button variant="secondary" class="h-8 text-sm flex! items-center w-full">
                <Edit class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                <span>View Details</span>
              </Button>
            </a>
            {#if recording.status === 'completed'}
              <Button
                onclick={() => handleUpload(recording.id)}
                class="h-8 text-sm flex! items-center">
                <Upload class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                <span>Upload</span>
              </Button>
              <Button
                onclick={() => handleProcess(recording.id)}
                class="h-8 text-sm flex! items-center"
                disabled={processing === recording.id}>
                {#if processing === recording.id}
                  <div
                    class="w-3.5 h-3.5 mr-1.5 border-2 border-t-transparent border-white rounded-full animate-spin">
                  </div>
                  <span>Processing...</span>
                {:else}
                  <Play class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                  <span>Process</span>
                {/if}
              </Button>
            {/if}
          </div>
        </div>
      </Card>
    {/each}
  </div>
</div>
