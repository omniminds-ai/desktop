import posthog from 'posthog-js';
import { browser } from '$app/environment';
// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load = async () => {
  if (browser) {
    posthog.init('phc_fj4VJCcM1CdHiNzhNDVi2LwClnEN1DG3zApjNiSJgld', {
      api_host: 'https://insi.viralmind.ai',
      person_profiles: 'identified_only', // or 'always' to create profiles for anonymous users as well
      capture_pageview: false,
      capture_exceptions: true
    });
  }
  return;
};
