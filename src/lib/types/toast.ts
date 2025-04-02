/**
 * Toast notification type definitions
 */

/**
 * Button configuration for toast notifications
 */
export interface ToastButton {
  text: string;
  action: () => void;
  variant?: 'primary' | 'secondary' | 'warning' | 'destroy' | 'green';
}

/**
 * Toast notification data structure
 */
export interface Toast {
  id: string;
  title: string;
  message: string;
  timeout?: number; // in milliseconds, undefined means no auto-dismiss
  button?: ToastButton;
}
