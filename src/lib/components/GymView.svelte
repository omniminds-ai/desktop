<script lang="ts">
import Card from './Card.svelte';
import { Brain, DollarSign, Palette, FileSpreadsheet } from 'lucide-svelte';
</script>

<div class="h-full">
  <div class="p-4">
      <div class="mb-12">
        <h2 class="text-2xl font-title text-[var(--vm-secondary-300)] mb-6">Start Training</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
          <a href="/app/gym/chat?prompt=Free%20Race" class="block">
            <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
            <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4">
              <div class="w-10 h-10 sm:w-12 sm:h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center">
                <Brain class="w-5 h-5 sm:w-6 sm:h-6 text-[var(--vm-secondary-300)]" />
              </div>
              <div>
                <h3 class="text-lg sm:text-xl font-title">Free Races</h3>
                <p class="text-gray-600">Train AI and earn rewards</p>
              </div>
            </div>
            </Card>
          </a>

          <a href="/app/gym/chat?prompt=Staked%20Race" class="block">
            <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
            <div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-4">
              <div class="w-10 h-10 sm:w-12 sm:h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center">
                <DollarSign class="w-5 h-5 sm:w-6 sm:h-6 text-[var(--vm-secondary-300)]" />
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
        <h2 class="text-2xl font-title text-[var(--vm-secondary-300)] mb-6">Featured Races</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
          <a href="/app/gym/chat?prompt=Random%20Challenge" class="block">
            <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
            <div class="flex flex-col h-full">
              <div class="w-10 h-10 sm:w-12 sm:h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-3 sm:mb-4">
                <Brain class="w-5 h-5 sm:w-6 sm:h-6 text-[var(--vm-secondary-300)]" />
              </div>
              <h3 class="text-lg sm:text-xl font-title mb-2">Random Challenge</h3>
              <p class="text-gray-600 mb-6">Our AI guides you through random desktop tasks</p>
              <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
                Join Race
              </button>
            </div>
            </Card>
          </a>

          <a href="/app/gym/chat?prompt=Paint%20Buddy" class="block">
            <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
            <div class="flex flex-col h-full">
              <div class="w-10 h-10 sm:w-12 sm:h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-3 sm:mb-4">
                <Palette class="w-5 h-5 sm:w-6 sm:h-6 text-[var(--vm-secondary-300)]" />
              </div>
              <h3 class="text-lg sm:text-xl font-title mb-2">Paint Buddy</h3>
              <p class="text-gray-600 mb-6">Let our AI guide you through artistic adventures</p>
              <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
                Join Race
              </button>
            </div>
            </Card>
          </a>

          <a href="/app/gym/chat?prompt=Office%20Helper" class="block">
            <Card padding="lg" className="hover:scale-[1.02] transition-transform cursor-pointer">
            <div class="flex flex-col h-full">
              <div class="w-10 h-10 sm:w-12 sm:h-12 bg-[var(--vm-secondary-100/10)] rounded-lg flex items-center justify-center mb-3 sm:mb-4">
                <FileSpreadsheet class="w-5 h-5 sm:w-6 sm:h-6 text-[var(--vm-secondary-300)]" />
              </div>
              <h3 class="text-lg sm:text-xl font-title mb-2">Office Helper</h3>
              <p class="text-gray-600 mb-6">Follow along with spreadsheet and document tasks</p>
              <button class="mt-auto px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded-lg hover:bg-[var(--vm-secondary-400)] transition-colors">
                Join Race
              </button>
            </div>
            </Card>
          </a>
        </div>
      </div>
  </div>
</div>
