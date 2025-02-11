<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Search, Upload, Edit, Download } from 'lucide-svelte';
  import GymHeader from '$lib/components/gym/GymHeader.svelte';

  let searchQuery = '';
  let sortOrder: 'newest' | 'oldest' = 'newest';
  let races = [
    {
      id: '1',
      title: 'AI Training Session',
      description: 'Helped improve language understanding',
      tokens: 1500,
      duration_minutes: 25,
      reward: {
        amount: 50,
        ticker: 'VIRAL'
      },
      status: 'unclaimed'
    }
  ];

  $: filteredRaces = races
    .filter(race => 
      race.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      race.description.toLowerCase().includes(searchQuery.toLowerCase())
    )
    .sort((a, b) => {
      if (sortOrder === 'newest') {
        return (b.id as any) - (a.id as any);
      } else {
        return (a.id as any) - (b.id as any);
      }
    });

  function handleUpload(raceId: string) {
    console.log('Uploading race:', raceId);
    // Implement upload logic here
  }
</script>

<div class="h-full">
  <div class="p-4">
    <GymHeader title="Race History" />
    
    <div class="flex flex-col sm:flex-row gap-4 mb-3">
      <div class="relative flex-grow">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search races..."
          class="w-full pl-10 pr-4 py-2 bg-white border-2 border-gray-200 rounded-lg focus:outline-none focus:border-secondary-300 text-gray-800"
        />
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
      </div>
      
      <select
        bind:value={sortOrder}
        class="px-4 py-2 bg-white border-2 border-gray-200 rounded-lg focus:outline-none focus:border-secondary-300 text-gray-800"
      >
        <option value="newest">Newest First</option>
        <option value="oldest">Oldest First</option>
      </select>
    </div>
  </div>

  <div class="grid gap-4 px-2">
    {#each filteredRaces as race}
      <Card padding="lg" className="border-2">
        <div class="flex flex-col sm:flex-row justify-between gap-4">
          <div>
            <h3 class="text-xl font-title mb-2">{race.title}</h3>
            <p class="text-gray-700 mb-3">{race.description}</p>
            <div class="flex flex-wrap gap-4 text-sm text-gray-600">
              <span>{race.tokens} tokens</span>
              <span>{race.duration_minutes} minutes</span>
              <span>Reward: {race.reward.amount} ${race.reward.ticker}</span>
              {#if race.status === 'unclaimed'}
                <span class="text-yellow-500">(Unclaimed)</span>
              {/if}
            </div>
          </div>
          
          <div class="flex flex-col gap-2 w-32">
            <Button variant="secondary" onclick={() => console.log('Edit race:', race.id)} class="h-8 text-sm flex! items-center">
              <Edit class="w-3.5 h-3.5 mr-1.5 shrink-0" />
              <span>Edit</span>
            </Button>
            <Button variant="secondary" onclick={() => console.log('Export race:', race.id)} class="h-8 text-sm flex! items-center">
              <Download class="w-3.5 h-3.5 mr-1.5 shrink-0" />
              <span>Export</span>
            </Button>
            {#if race.status === 'unclaimed'}
              <Button onclick={() => handleUpload(race.id)} class="h-8 text-sm flex! items-center">
                <Upload class="w-3.5 h-3.5 mr-1.5 shrink-0" />
                <span>Upload</span>
              </Button>
            {/if}
          </div>
        </div>
      </Card>
    {/each}
  </div>
</div>
