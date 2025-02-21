<script lang="ts">
  import { goto } from '$app/navigation';
  import Button from '../Button.svelte';
  import Card from '../Card.svelte';
  import { ExternalLink } from 'lucide-svelte';

  export let recordingId: string;
</script>

<Card variant="secondary" padding="sm" className="w-auto! shadow-sm space-y-4 relative bg-black/5">
  <div class="flex gap-4 items-center justify-between">
    <div class="text-sm font-medium">Recording saved!</div>
    <Button
      variant="primary"
      class="text-xs! py-1! px-2! flex! items-center! gap-1!"
      onclick={() => goto(`/app/gym/history/recording?id=${recordingId}`)}>
      View Details
      <ExternalLink size={14} />
    </Button>
  </div>
</Card>
