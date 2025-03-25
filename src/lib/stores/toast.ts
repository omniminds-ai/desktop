import { writable } from 'svelte/store';

export interface ToastButton {
  text: string;
  action: () => void;
  variant?: 'primary' | 'secondary' | 'warning' | 'destroy' | 'green';
}

export interface Toast {
  id: string;
  title: string;
  message: string;
  timeout?: number; // in milliseconds, undefined means no auto-dismiss
  button?: ToastButton;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  return {
    subscribe,
    add: (toast: Omit<Toast, 'id'>) => {
      const id = crypto.randomUUID();
      update(toasts => [...toasts, { ...toast, id }]);
      
      // Auto-dismiss toast after timeout if specified
      if (toast.timeout) {
        setTimeout(() => {
          update(toasts => toasts.filter(t => t.id !== id));
        }, toast.timeout);
      }
      
      return id;
    },
    remove: (id: string) => {
      update(toasts => toasts.filter(toast => toast.id !== id));
    },
    clear: () => {
      update(() => []);
    }
  };
}

export const toasts = createToastStore();
