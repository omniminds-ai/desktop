<script lang="ts">
  import Card from './Card.svelte';
  import { onMount } from 'svelte';
  import { getRaces } from '$lib/api/forge';
  import type { Race } from '$lib/types/gym';
  import GymHeader from './gym/GymHeader.svelte';

  const freeRacePrompts = [
    "Hey, I'd like to help train AI and maybe earn some $VIRAL. What kind of tasks can I help with?",
    "Hi! I'm interested in contributing to AI training and earning rewards. What should I do?",
    "I want to help improve AI and earn some $VIRAL tokens. Where do I start?",
    "Looking to contribute to AI training and get rewarded. What tasks are available?",
    "Hey there! Ready to help train AI and earn some rewards. What's next?"
  ];
  const randomFreeRacePrompt = freeRacePrompts[Math.floor(Math.random() * freeRacePrompts.length)];
  import {
    Brain,
    DollarSign,
    Globe,
    FileText,
    Mail,
    Film,
    Code,
    Briefcase,
    Image
  } from 'lucide-svelte';

  let races: Race[] = [];
  const iconMap = {
    Brain,
    DollarSign,
    Globe,
    FileText,
    Mail,
    Film,
    Code,
    Briefcase,
    Image
  };

  onMount(async () => {
    try {
      races = await getRaces();
    } catch (error) {
      console.error('Failed to fetch races:', error);
    }
  });

  function getIcon(iconName: string) {
    return iconMap[iconName as keyof typeof iconMap] || Brain;
  }
</script>

<div class="h-full">
  <div class="p-4">
    <div class="mb-12">
      <GymHeader title="Start Training" />
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
        <a href="/app/gym/chat?prompt={encodeURIComponent(randomFreeRacePrompt)}" class="block">
          <Card
            padding="lg"
            className="hover:scale-[1.02] hover:border-secondary-300 border-2 transition-all cursor-pointer h-full">
            <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4">
              <div
                class="w-10 h-10 sm:w-12 sm:h-12 bg-secondary-100/10 rounded-lg flex items-center justify-center">
                <Brain class="w-5 h-5 sm:w-6 sm:h-6 text-secondary-300" />
              </div>
              <div>
                <h3 class="text-lg sm:text-xl font-title">Free Races</h3>
                <p class="text-gray-600">Train AI and earn rewards</p>
              </div>
            </div>
          </Card>
        </a>

        <a href="/app/gym/chat?prompt=Staked%20Race" class="block">
          <Card
            padding="lg"
            className="hover:scale-[1.02] hover:border-secondary-300 border-2 transition-all cursor-pointer h-full">
            <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4">
              <div
                class="w-10 h-10 sm:w-12 sm:h-12 bg-secondary-100/10 rounded-lg flex items-center justify-center">
                <DollarSign class="w-5 h-5 sm:w-6 sm:h-6 text-secondary-300" />
              </div>
              <div>
                <h3 class="text-lg sm:text-xl font-title">Staked Races</h3>
                <p class="text-gray-600">High stakes, high rewards</p>
              </div>
            </div>
          </Card>
        </a>
      </div>
    </div>

    <div>
      <h2 class="text-2xl font-title text-secondary-300 mb-6">Featured Races</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
        {#each races as race}
          <a href="/app/gym/chat?prompt={encodeURIComponent(race.agent_prompt)}" class="block h-full">
            <Card
              padding="lg"
              className="hover:scale-[1.02] hover:border-secondary-300 border-2 transition-all cursor-pointer h-full">
              <div class="flex flex-col h-full">
                <div
                  class="w-10 h-10 sm:w-12 sm:h-12 bg-secondary-100/10 rounded-lg flex items-center justify-center mb-3 sm:mb-4">
                  <svelte:component
                    this={getIcon(race.icon)}
                    class="w-5 h-5 sm:w-6 sm:h-6 text-secondary-300" />
                </div>
                <h3 class="text-lg sm:text-xl font-title mb-2">{race.title}</h3>
                <p class="text-gray-600 flex-grow">{race.description}</p>
              </div>
            </Card>
          </a>
        {/each}
      </div>
    </div>
  </div>
</div>
