<script lang="ts">
  import {
    CrownIcon,
    PencilRuler,
    Rocket,
    ArrowLeftRight,
    Users,
    FileText,
    BrainCog,
    Search,
    Bell,
    MessageSquare,
    RotateCcw
  } from 'lucide-svelte';
  import Input from '$lib/components/form/Input.svelte';
  import WalletButton from './WalletButton.svelte';
  import { stopRecording } from '$lib/api/endpoints/gym';
  import { RecordingState } from '$lib/types/gym';
  import { recordingState } from '$lib/stores/recording';
  import {searchStore} from '$lib/stores/search';

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

  let debounced = false;
  let debouncedSearch = "";

  function handleSearch(event) {
    if (debouncedSearch.toLowerCase() != event.target.value.toLowerCase()) {
      debouncedSearch = event.target.value;
      debounce()
    }
  }

  const debounce = () => {
    if (debounced == false) {
      debounced = true
      setTimeout(() => {
        $searchStore = debouncedSearch;
        debounced = false;
      }, 500)
    }
  }
</script>

<div class="flex w-full items-center justify-between py-4 max-w-7xl">
  <!-- Left side - Search -->
  <div class="flex items-center">
    <div class="relative">
      <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
      <input
        type="text"
        on:input={handleSearch}
        placeholder="Search..."
        class="w-80 rounded-full bg-gray-800/50 border border-gray-700/50 pl-10 pr-4 py-2.5 text-sm text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-purple-500/50 focus:border-purple-500/50" />
    </div>
  </div>

  <div class="flex items-center gap-4">
    <button class="p-2 rounded-lg hover:bg-gray-800/50 transition-colors">
      <Bell class="h-5 w-5 text-gray-400 hover:text-white" />
    </button>

    <div class="flex items-center gap-2">
      <div class="flex items-center gap-1">
        <WalletButton />
      </div>
    </div>
  </div>
</div>
