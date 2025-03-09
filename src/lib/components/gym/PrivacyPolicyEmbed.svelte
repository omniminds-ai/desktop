<script lang="ts">
  import PrivacyPolicy from '../PrivacyPolicy.svelte';

  let { privacyAccepted, class: className }: { privacyAccepted: () => boolean; class?: string } =
    $props();
</script>

{#if privacyAccepted()}
  <a
    href="https://github.com/viralmind-ai/desktop/blob/main/PRIVACY.md"
    class="text-secondary-300 hover:underline"
    target="_blank">
    View Privacy Policy
  </a>
{:else}
  <div class={className}>
    <PrivacyPolicy class={className} />
  </div>
{/if}
