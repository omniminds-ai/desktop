<script lang="ts">
  import Card from '../Card.svelte';
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

  function getFileName(submission: PoolSubmission): string {
    if (submission.files && submission.files.length > 0) {
      // Extract filename from file path or s3Key
      const filePath = submission.files[0].file || submission.files[0].s3Key || '';
      const parts = filePath.split('/');
      return parts[parts.length - 1] || 'Unknown File';
    }
    return 'No File';
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
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Date
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Filename
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Status
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Reward
          </th>
          <th
            class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
            Actions
          </th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each submissions as submission}
          <tr>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {formatDate(submission.createdAt)}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {getFileName(submission)}
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span
                class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full
                {submission.status === 'COMPLETED'
                  ? 'bg-green-100 text-green-800'
                  : submission.status === 'FAILED'
                    ? 'bg-red-100 text-red-800'
                    : 'bg-yellow-100 text-yellow-800'}">
                {submission.status}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              {submission.reward || 0}
              {pool.token.symbol}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
              {#if submission.grade_result}
                <a href="#" class="text-secondary-600 hover:text-secondary-900">View Details</a>
              {:else}
                <span class="text-gray-400">No Details</span>
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
