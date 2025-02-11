import { writable } from 'svelte/store';

// Initialize from localStorage if available, otherwise false
const storedAcceptance = typeof localStorage !== 'undefined' && localStorage.getItem('privacyAccepted');
export const privacyAccepted = writable<boolean>(storedAcceptance === 'true');

// Subscribe to changes and update localStorage
privacyAccepted.subscribe((value) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('privacyAccepted', value.toString());
  }
});
