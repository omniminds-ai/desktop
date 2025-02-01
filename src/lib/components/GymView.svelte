<script lang="ts">
import Card from './Card.svelte';
let showCustomTask = $state(false);

const taskCategories = [
  {
    name: 'Desktop Tasks',
    description: 'File management, system operations, and general desktop tasks',
    multiplier: 1.0,
    demand: 'High',
    examples: ['Rename files in bulk', 'Organize folders', 'Create desktop shortcuts']
  },
  {
    name: 'Office Suite',
    description: 'Excel, Word, PowerPoint, and other office applications',
    multiplier: 1.2,
    demand: 'Very High',
    examples: ['Format Excel spreadsheet', 'Create PowerPoint template', 'Mail merge in Word']
  },
  {
    name: 'Creative Tools',
    description: 'Paint, Photo editing, and design applications',
    multiplier: 1.5,
    demand: 'Medium',
    examples: ['Create logo in Paint', 'Basic photo edits', 'Design social media banner']
  },
  {
    name: 'Custom Tasks',
    description: 'Propose your own desktop automation task',
    multiplier: '1.0-2.0',
    demand: 'Varies',
    examples: ['Your creative desktop automation ideas']
  }
];
</script>

<div class="h-full space-y-6 p-4">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl">Training Gym</h2>
    <div class="flex items-center gap-2">
      <span class="text-green-500 font-medium">Balance: 245 $VIRAL</span>
      <button class="px-4 py-2 bg-green-500 text-white rounded-lg">Withdraw</button>
    </div>
  </div>

  {#if showCustomTask}
    <Card className="space-y-4">
      <div class="flex justify-between items-center">
        <h3 class="text-lg font-semibold">Submit Custom Task</h3>
        <button 
          onclick={() => showCustomTask = false}
          class="text-gray-500 hover:text-gray-700"
        >
          Cancel
        </button>
      </div>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Task Description
          </label>
          <textarea 
            class="w-full p-2 rounded-lg bg-white/50 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)]"
            rows="3"
            placeholder="Describe your desktop automation task..."
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Category
          </label>
          <select class="w-full p-2 rounded-lg bg-white/50 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)]">
            <option>Desktop Management</option>
            <option>Office Applications</option>
            <option>Creative Tools</option>
            <option>Other</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Expected Duration (minutes)
          </label>
          <input 
            type="number" 
            min="1"
            max="60"
            class="w-full p-2 rounded-lg bg-white/50 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)]"
          />
        </div>
        <button class="w-full px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg">
          Submit for Review
        </button>
      </div>
    </Card>
  {:else}
    <div class="grid grid-cols-2 gap-6">
      {#each taskCategories as category}
        <Card padding="lg">
          <div>
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-lg">{category.name}</h3>
              <div class="flex items-center gap-2">
                <span class="text-sm text-gray-500">Demand:</span>
                <span class={`text-sm font-medium ${
                  category.demand === 'High' ? 'text-green-500' :
                  category.demand === 'Very High' ? 'text-[var(--vm-secondary-300)]' :
                  category.demand === 'Medium' ? 'text-yellow-500' :
                  'text-gray-500'
                }`}>{category.demand}</span>
              </div>
            </div>
            <p class="text-gray-600 mb-4">{category.description}</p>
            <div class="space-y-2 mb-4">
              {#each category.examples as example}
                <div class="text-sm text-gray-500 flex items-center gap-2">
                  <div class="w-1 h-1 rounded-full bg-gray-400"></div>
                  {example}
                </div>
              {/each}
            </div>
            <div class="flex justify-between items-center">
              <div class="text-sm">
                <span class="text-gray-500">Reward Multiplier: </span>
                <span class="font-medium text-[var(--vm-secondary-300)]">x{category.multiplier}</span>
              </div>
              {#if category.name === 'Custom Tasks'}
                <button 
                  onclick={() => showCustomTask = true}
                  class="px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg"
                >
                  Propose Task
                </button>
              {:else}
                <button class="px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg">
                  Start Task
                </button>
              {/if}
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}

  <div class="grid grid-cols-2 gap-4">
    <Card padding="sm">
      <h4 class="mb-2">Daily Stats</h4>
      <div class="text-2xl font-bold">12 Tasks</div>
      <div class="text-green-500">+180 $VIRAL</div>
    </Card>
    <Card padding="sm">
      <h4 class="mb-2">Leaderboard Rank</h4>
      <div class="text-2xl font-bold">#42</div>
      <div class="text-gray-500">Top 5%</div>
    </Card>
  </div>
</div>
