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
  let errorData: any;
  let errorMessage = `${response.status} ${response.statusText}`;

  try {
    // Try to parse error data from response
    errorData = await response.json();
    if (errorData.message || errorData.error) {
      errorMessage = errorData.message || errorData.error;
    }
  } catch (e) {
    // If parsing fails, use the status text
    errorData = null;
  }

  throw new ApiError(
    response.status,
    response.statusText,
    errorMessage,
    errorData
  );
}

/**
 * Utility to create a timeout error
 * @param timeoutMs Timeout in milliseconds
 * @returns ApiError instance
 */
export function createTimeoutError(timeoutMs: number): ApiError {
  return new ApiError(
    408,
    'Request Timeout',
    `Request timed out after ${timeoutMs}ms`,
    { timeoutMs }
  );
}
