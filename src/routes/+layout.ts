import { browser } from '$app/environment';
// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;
import {attachConsole} from '@tauri-apps/plugin-log'


export const load = async () => {
 //TODO: setup analytics
    attachConsole().then(detach => {
        //do nothing
    })
};
