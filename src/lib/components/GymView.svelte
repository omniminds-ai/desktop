<script lang="ts">
import Card from './Card.svelte';
import { Brain, DollarSign, Palette, FileSpreadsheet } from 'lucide-svelte';
</script>

<div class="h-full p-4">
  <div class="flex justify-between items-center">
    <!-- <h2 class="text-2xl">Viral Node</h2> -->
    <!-- <div class="flex items-center gap-2">
      <span class="text-green-500 font-medium">Earned: 1,245 $VIRAL</span>
      <button class="px-4 py-2 bg-green-500 text-white rounded-lg">Claim</button>
    </div> -->
  </div>

  <div class="mb-12">
    <h2 class="text-2xl font-title text-[var(--vm-secondary-300)] mb-6">Start Training</h2>
    <div class="grid grid-cols-2 gap-6">
      <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center">
            <Brain class="w-6 h-6 text-[var(--vm-secondary-300)]" />
          </div>
          <div>
            <h3 class="text-xl font-title">Free Races</h3>
            <p class="text-gray-600">Train AI and earn rewards</p>
          </div>
        </div>
      </Card>

      <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center">
            <DollarSign class="w-6 h-6 text-[var(--vm-secondary-300)]" />
          </div>
          <div>
            <h3 class="text-xl font-title">Staked Races</h3>
            <p class="text-gray-600">High stakes, high rewards</p>
          </div>
        </div>
      </Card>
    </div>
  </div>

  <div>
    <h2 class="text-2xl font-title text-[var(--vm-secondary-300)] mb-6">Featured Races</h2>
    <div class="grid grid-cols-3 gap-6">
      <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
        <div class="flex flex-col h-full">
          <div class="w-12 h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-4">
            <Brain class="w-6 h-6 text-[var(--vm-secondary-300)]" />
          </div>
          <h3 class="text-xl font-title mb-2">Random Challenge</h3>
          <p class="text-gray-600 mb-6">Our AI guides you through random Linux tasks</p>
          <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
            Join Race
          </button>
        </div>
      </Card>

      <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
        <div class="flex flex-col h-full">
          <div class="w-12 h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-4">
            <Palette class="w-6 h-6 text-[var(--vm-secondary-300)]" />
          </div>
          <h3 class="text-xl font-title mb-2">Paint Buddy</h3>
          <p class="text-gray-600 mb-6">Let our AI guide your browser-based artistic adventures</p>
          <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
            Join Race
          </button>
        </div>
      </Card>

      <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
        <div class="flex flex-col h-full">
          <div class="w-12 h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-4">
            <FileSpreadsheet class="w-6 h-6 text-[var(--vm-secondary-300)]" />
          </div>
          <h3 class="text-xl font-title mb-2">Office Helper</h3>
          <p class="text-gray-600 mb-6">Follow along with spreadsheet and document tasks</p>
          <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
            Join Race
          </button>
        </div>
      </Card>
    </div>
  </div>
</div>
