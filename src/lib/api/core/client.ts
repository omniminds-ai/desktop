import { API_URL } from '$lib/utils/platform';
import { get } from 'svelte/store';
import { connectionToken } from '$lib/stores/wallet';
import { ApiError, handleApiError } from './errors';

export type RequestOptions = {
  requiresAuth?: boolean;
  headers?: Record<string, string>;
};

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl = `${API_URL}/api`) {
    this.baseUrl = baseUrl;
  }

  private getAuthToken(): string {
    return get(connectionToken) || '';
  }

  private getHeaders(options: RequestOptions = {}): Headers {
    const headers = new Headers(options.headers);

    if (options.requiresAuth) {
      headers.set('x-connect-token', this.getAuthToken());
    }

    return headers;
  }

  async get<T>(
    endpoint: string,
    params?: Record<string, any>,
    options: RequestOptions = {}
  ): Promise<T> {
    const url = new URL(`${this.baseUrl}${endpoint}`);

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            url.searchParams.set(key, value.toString());
          } else {
            url.searchParams.set(key, String(value));
          }
        }
      });
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: this.getHeaders(options)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json();
  }

  async post<T>(endpoint: string, data?: any, options: RequestOptions = {}): Promise<T> {
    const headers = this.getHeaders(options);

    if (data && !(data instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      headers,
      body: data instanceof FormData ? data : JSON.stringify(data)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json();
  }

  async put<T>(endpoint: string, data?: any, options: RequestOptions = {}): Promise<T> {
    const headers = this.getHeaders(options);

    if (data && !(data instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'PUT',
      headers,
      body: data instanceof FormData ? data : JSON.stringify(data)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json();
  }

  async delete<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: 'DELETE',
      headers: this.getHeaders(options)
    });

    if (!response.ok) {
      handleApiError(response);
    }

    return response.json();
  }
}

// Create and export a singleton instance
export const apiClient = new ApiClient();
