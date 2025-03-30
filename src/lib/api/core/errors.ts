import type { ApiResponse } from './client';

/**
 * Custom API error class
 */
export class ApiError extends Error {
  status: number;
  statusText: string;
  data?: any;

  constructor(status: number, statusText: string, message: string, data?: any) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.statusText = statusText;
    this.data = data;
  }
}

/**
 * Handles API errors consistently
 * @param response Fetch response object
 * @returns Promise that rejects with an ApiError
 */
export async function handleApiError(response: Response): Promise<never> {
  let data: ApiResponse<any> | null;
  let errorMessage = `${response.status} ${response.statusText}`;

  try {
    // Try to parse error data from response
    data = await response.json();
    if (data?.error) {
      errorMessage = `${data.error.code}: ${data.error.message}${
        data.error.details ? `\n${data.error.details}` : ''
      }`;
    }
  } catch (e) {
    // If parsing fails, use the status text
    data = null;
  }

  throw new ApiError(response.status, response.statusText, errorMessage, data);
}

/**
 * Utility to create a timeout error
 * @param timeoutMs Timeout in milliseconds
 * @returns ApiError instance
 */
export function createTimeoutError(timeoutMs: number): ApiError {
  return new ApiError(408, 'Request Timeout', `Request timed out after ${timeoutMs}ms`, {
    timeoutMs
  });
}
