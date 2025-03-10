import { API_URL } from '$lib/utils';
import { writable, get } from 'svelte/store';

// Initialize from localStorage if available
const storedAddress =
  typeof localStorage !== 'undefined' ? localStorage.getItem('walletAddress') : null;
const storedToken =
  typeof localStorage !== 'undefined' ? localStorage.getItem('connectionToken') : null;

export const walletAddress = writable<string | null>(storedAddress);
export const connectionToken = writable<string | null>(storedToken);
export const isConnecting = writable(false);

// Subscribe to changes and save to localStorage
if (typeof localStorage !== 'undefined') {
  walletAddress.subscribe((value) => {
    if (value) {
      localStorage.setItem('walletAddress', value);
    } else {
      localStorage.removeItem('walletAddress');
    }
  });
  
  connectionToken.subscribe((value) => {
    if (value) {
      localStorage.setItem('connectionToken', value);
    } else {
      localStorage.removeItem('connectionToken');
    }
  });
}

function generateToken() {
  return Math.random().toString(36).substring(2) + Date.now().toString(36);
}

export function getConnectionUrl() {
  let token = get(connectionToken);
  if (!token) {
    token = generateToken();
    connectionToken.set(token);
  }
  return `${API_URL}/connect?token=${token}`;
}

let currentInterval: number | null = null;

export function startPolling() {
  const token = get(connectionToken);
  if (!token) return;

  // Clear any existing interval
  if (currentInterval) {
    clearInterval(currentInterval);
  }

  isConnecting.set(true);

  // Reset "Connecting..." text after 5 seconds
  setTimeout(() => {
    isConnecting.set(false);
  }, 5000);

  // Poll for connection status
  currentInterval = setInterval(async () => {
    try {
      const response = await fetch(`${API_URL}/api/forge/check-connection?token=${token}`);
      const data = await response.json();

      if (data.connected) {
        walletAddress.set(data.address);
        if (currentInterval) {
          clearInterval(currentInterval);
          currentInterval = null;
        }
      }
    } catch (error) {
      console.error('Failed to check connection:', error);
    }
  }, 1000);
}

export function disconnectWallet() {
  walletAddress.set(null);
  connectionToken.set(null);
  if (currentInterval) {
    clearInterval(currentInterval);
    currentInterval = null;
  }
}
