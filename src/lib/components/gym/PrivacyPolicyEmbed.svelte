<script lang="ts">
  import PrivacyPolicy from "../PrivacyPolicy.svelte";

  let { privacyAccepted }: { privacyAccepted: () => boolean } = $props();
</script>

{#if privacyAccepted()}
  <a
    href="https://github.com/viralmind-ai/gym-desktop/blob/main/PRIVACY.md"
    class="text-secondary-300 hover:underline"
    target="_blank"
  >
    View Privacy Policy
  </a>
{:else}
  <PrivacyPolicy />
{/if}
