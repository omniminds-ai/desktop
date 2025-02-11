<script lang="ts">
import GymChat from '$lib/components/gym/GymChat.svelte';
import { page } from '$app/stores';

const prompt = $page.url.searchParams.get('prompt') || '';
</script>

<GymChat prompt={prompt} />
