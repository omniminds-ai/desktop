<script lang="ts">
  import Card from '../Card.svelte';
  import Button from '../Button.svelte';
  import { TrainingPoolStatus, type TrainingPool } from '$lib/types/forge';
  import { getPoolSubmissions, type PoolSubmission } from '$lib/api/forge';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Download, File } from 'lucide-svelte';
  
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
  
  function getFileName(submission: PoolSubmission): string {
    if (submission.files && submission.files.length > 0) {
      // Extract filename from file path or s3Key
      const filePath = submission.files[0].file || submission.files[0].s3Key || '';
      const parts = filePath.split('/');
      return parts[parts.length - 1] || 'Unknown File';
    }
    return 'No File';
  }
  
  // Get S3 URL from key
  function getS3Url(s3Key: string): string {
    return `https://training-gym.s3.us-east-2.amazonaws.com/${s3Key}`;
  }
  
  let downloadingFiles = new Set<string>();
  
  // Sanitize a string to remove all characters except alphanumerics, dashes, colons, slashes, dots, and numbers
  function sanitizeString(str: string): string {
    return str.replace(/[^a-zA-Z0-9\-:\/\.\d]/g, '');
  }

  // Download all files for a submission
  async function downloadFiles(submission: PoolSubmission): Promise<void> {
    if (!submission.files || submission.files.length === 0) {
      console.error('No files to download');
      return;
    }
    
    // Use submission id for folder name
    const submissionId = sanitizeString(submission._id);
    downloadingFiles.add(submissionId);
    
    try {
      // Get download path from settings
      const settings = await invoke<any>('get_settings');
      const baseDownloadPath = settings?.engines?.downloads_path || '~/.viralmind/downloads';
      
      // Get the sanitized gym name for folder structure
      const gymName = sanitizeString(pool.name);
      
      // Create a specific folder hierarchy: downloads/submission-id
      const submissionDownloadPath = `${baseDownloadPath}/${submissionId}`;
      
      // Create the directory
      const mkdirCmd = `mkdir -p ${submissionDownloadPath}`;
      await invoke<string>('run_command', { command: mkdirCmd });
      
      // Download each file
      for (let i = 0; i < submission.files.length; i++) {
        const file = submission.files[i];
        if (!file.s3Key) continue;
        
        const s3Key = file.s3Key;
        
        // Extract filename from S3 key
        const fileParts = s3Key.split('/');
        const filename = fileParts[fileParts.length - 1];
        
        // Create download URL
        const url = getS3Url(s3Key);
        
        // Sanitize filename and URL
        const sanitizedFilename = sanitizeString(filename);
        const sanitizedUrl = sanitizeString(url);
        
        // Download file with curl
        const curlCmd = `curl -L "${sanitizedUrl}" -o ${submissionDownloadPath}/${sanitizedFilename}`;
        
        const jobId = await invoke<string>('run_command', {
          command: curlCmd
        });
        
        // Poll for job completion
        let completed = false;
        while (!completed) {
          const job = await invoke<any>('get_job', { jobId });
          if (job.status !== 'Running') {
            if (job.status === 'Failed') {
              throw new Error(job.error || `Download failed for file ${i + 1}`);
            }
            completed = true;
          } else {
            // Wait before polling again
            await new Promise(resolve => setTimeout(resolve, 500));
          }
        }
      }
      
      // Notify user of successful download
      alert(`Downloaded ${submission.files.length} files to ${submissionDownloadPath}`);
      
      
    } catch (error) {
      console.error('Failed to download file:', error);
      alert(`Download failed: ${error}`);
    } finally {
      downloadingFiles.delete(submissionId);
    }
  }
</script>

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
      <button class="mt-2 text-secondary-600 hover:text-secondary-900 text-sm" 
        on:click={() => {
          loading = true;
          getPoolSubmissions(pool._id)
            .then(data => {
              submissions = data;
              error = null;
            })
            .catch(err => {
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
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Reward</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Files</th>
          <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Details</th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each submissions as submission}
          <tr>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{formatDate(submission.createdAt)}</td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                {submission.status === 'completed' 
                  ? 'bg-green-100 text-green-800' 
                  : submission.status === 'failed'
                    ? 'bg-red-100 text-red-800'
                    : 'bg-yellow-100 text-yellow-800'}">
                {submission.status}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{submission.reward || 0} {pool.token.symbol}</td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {#if submission.files && submission.files.length > 0}
                <div class="flex items-center gap-2">
                  <File size={16} />
                  <span class="truncate max-w-40 inline-block">{submission.files.length} {submission.files.length === 1 ? 'File' : 'Files'}</span>
                  <Button 
                    variant="secondary" 
                    disabled={downloadingFiles.has(submission._id)}
                    onclick={() => downloadFiles(submission)}
                    class="!p-1 min-w-8 h-8 flex items-center justify-center"
                  >
                    {#if downloadingFiles.has(submission._id)}
                      <div class="animate-spin">⚙️</div>
                    {:else}
                      <Download size={16} />
                    {/if}
                  </Button>
                </div>
              {:else}
                <span class="text-gray-400">No files</span>
              {/if}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
              {#if submission.meta}
                <div class="max-w-40">
                  <p class="text-xs text-gray-700 font-semibold truncate">
                    {submission.meta.title || (submission.meta.quest?.title) || 'Untitled'}
                  </p>
                  {#if submission.meta.quest}
                    <p class="text-xs text-gray-500 truncate">
                      {submission.meta.quest.app || 'Unknown App'}
                    </p>
                  {/if}
                </div>
              {:else if submission.grade_result}
                <a href="#" class="text-secondary-600 hover:text-secondary-900">View Details</a>
              {:else}
                <span class="text-gray-400">No details</span>
              {/if}
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
