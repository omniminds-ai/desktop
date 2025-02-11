import { writable } from 'svelte/store';

// Load initial blacklist from localStorage if available
const storedBlacklist = typeof localStorage !== 'undefined' ? localStorage.getItem('appBlacklist') : null;
const initialBlacklist = storedBlacklist ? JSON.parse(storedBlacklist) : [];

// Create the store
const blacklistedApps = writable<string[]>(initialBlacklist);

// Subscribe to changes and update localStorage
if (typeof window !== 'undefined') {
  blacklistedApps.subscribe(value => {
    localStorage.setItem('appBlacklist', JSON.stringify(value));
  });
}

export { blacklistedApps };
