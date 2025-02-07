<script lang="ts">
  import GymView from "$lib/components/GymView.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  onMount(async () => {
    await invoke("request_record_perms");
    await invoke("request_input_perms");
  });
</script>

<GymView />
