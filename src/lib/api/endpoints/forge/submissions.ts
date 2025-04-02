import { apiClient } from '../../core/client';
import type { PoolSubmission, SubmissionStatus } from '$lib/types/forge';

/**
 * Get the status of a submission
 * @param submissionId The ID of the submission
 * @returns Promise resolving to the submission status
 */
export async function getSubmissionStatus(submissionId: string): Promise<SubmissionStatus> {
  return apiClient.get<SubmissionStatus>(
    `/forge/submissions/${submissionId}`,
    {},
    { requiresAuth: true }
  );
}

/**
 * List all submissions for the current user
 * @returns Promise resolving to an array of submission statuses
 */
export async function listSubmissions(): Promise<SubmissionStatus[]> {
  return apiClient.get<SubmissionStatus[]>('/forge/submissions/user', {}, { requiresAuth: true });
}

/**
 * Get submissions for a specific pool
 * @param poolId The ID of the pool
 * @returns Promise resolving to an array of pool submissions
 */
export async function getPoolSubmissions(poolId: string): Promise<PoolSubmission[]> {
  return apiClient.get<PoolSubmission[]>(
    `/forge/submissions/pool/${poolId}`,
    {},
    { requiresAuth: true }
  );
}
