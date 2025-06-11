<script lang="ts">
  import {
    CrownIcon,
    PencilRuler,
    Rocket,
    ArrowLeftRight,
    Users,
    FileText,
    BrainCog
  } from 'lucide-svelte';
  import Input from '$lib/components/form/Input.svelte';
  import { stopRecording } from '$lib/api/endpoints/gym';
  import { RecordingState } from '$lib/types/gym';
  import { recordingState } from '$lib/stores/recording';

  const earnButtons = [
    { path: '/app/gym', icon: CrownIcon, label: 'The Arena' },
    { path: '/app/leaderboards', icon: PencilRuler, label: 'Creator Hub' },
    { path: '/app/launch-bay', icon: Rocket, label: 'LaunchBay' },
    { path: '/app/exchange', icon: ArrowLeftRight, label: 'The Exchange' },
    { path: '/app/my-agents', icon: Users, label: 'My Agents ' },
    { path: '/app/omni-docs', icon: FileText, label: 'OmniDocs ' },
    { path: '/app/settings', icon: BrainCog, label: 'Settings ' }
  ];

  const buttons = [...earnButtons];
  // Recording state
  let recordingLoading = false;

  recordingState.subscribe((state) => {
    if (state === RecordingState.starting || state == RecordingState.saving) {
      recordingLoading = true;
    } else {
      recordingLoading = false;
    }
  });

  // Handle stop recording click
  async function handleStopRecording() {
    try {
      await stopRecording();
    } catch (error) {
      console.error('Failed to stop recording:', error);
    }
  }
</script>

<div class="flex w-full">
  <Input placeholder="Search" class="max-w-[300px] rounded-full" />
</div>
