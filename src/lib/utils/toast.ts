import { toasts, type ToastButton } from '$lib/stores/toast';

/**
 * Show a toast notification
 * @param title The title of the toast
 * @param message The message content of the toast
 * @param options Additional options for the toast
 * @returns The ID of the created toast
 */
export function showToast(
  title: string,
  message: string,
  options?: {
    timeout?: number;
    button?: {
      text: string;
      action: () => void;
      variant?: 'primary' | 'secondary' | 'warning' | 'destroy' | 'green';
    };
  }
) {
  return toasts.add({
    title,
    message,
    timeout: options?.timeout,
    button: options?.button
  });
}

/**
 * Remove a specific toast by ID
 * @param id The ID of the toast to remove
 */
export function removeToast(id: string) {
  toasts.remove(id);
}

/**
 * Clear all toast notifications
 */
export function clearToasts() {
  toasts.clear();
}
