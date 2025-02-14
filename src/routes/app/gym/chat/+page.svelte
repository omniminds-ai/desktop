<script lang="ts">
import GymChat2 from '$lib/components/gym/GymChat2.svelte';
import { page } from '$app/stores';

const prompt = $page.url.searchParams.get('prompt') || '';
const appParam = $page.url.searchParams.get('app');
let app = null;

try {
  if (appParam) {
    app = JSON.parse(decodeURIComponent(appParam));
  }
} catch (error) {
  console.error('Failed to parse app parameter:', error);
}
</script>

<GymChat2 {prompt} {app} />
