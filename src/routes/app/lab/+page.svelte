<script lang="ts">
  import Card from '$lib/components/Card.svelte';

  const trainedModels = [
    {
      name: 'FileOps Assistant',
      version: 'v1.2.0',
      description: 'Trained on 1,245 file management demonstrations'
    },
    {
      name: 'DataViz Pro',
      version: 'v2.0.1',
      description: 'Trained on 842 data visualization tasks'
    },
    {
      name: 'Code Assistant',
      version: 'v0.9.5',
      description: 'Trained on 567 coding demonstrations'
    }
  ];

  const trainingProgress = [
    {
      name: 'Email Assistant',
      progress: 87,
      description: 'Training on 450 email demonstrations'
    },
    {
      name: 'Calendar Manager',
      progress: 34,
      description: 'Training on 280 calendar demonstrations'
    }
  ];
</script>

<div class="h-full space-y-6 p-4">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold">Training Lab</h2>
    <button class="px-4 py-2 bg-secondary-300 text-white rounded-lg">New Experiment</button>
  </div>

  <div class="grid grid-cols-2 gap-6">
    <Card padding="lg">
      <h3 class="text-lg font-semibold mb-4">Trained Models</h3>
      <div class="space-y-4">
        {#each trainedModels as model}
          <Card variant="secondary" padding="sm">
            <div class="flex justify-between items-center mb-2">
              <div class="font-title">{model.name}</div>
              <div class="text-sm text-gray-500">{model.version}</div>
            </div>
            <p class="text-sm text-gray-600 mb-3">{model.description}</p>
            <div class="flex gap-2 mt-3">
              <button class="px-2 py-1 bg-secondary-300 text-white text-sm rounded">Chat</button>
              <button class="px-2 py-1 bg-gray-200 text-gray-700 text-sm rounded">Export</button>
              <button class="px-2 py-1 bg-gray-200 text-gray-700 text-sm rounded">Stats</button>
            </div>
          </Card>
        {/each}
      </div>
    </Card>

    <Card padding="lg">
      <h3 class="text-lg font-semibold mb-4">Training Progress</h3>
      <div class="space-y-4">
        {#each trainingProgress as model}
          <Card variant="secondary" padding="md">
            <div class="flex justify-between items-center mb-2">
              <div class="font-title">{model.name}</div>
              <div class="text-sm text-secondary-300">
                {model.progress}% complete
              </div>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="bg-secondary-300 h-2 rounded-full" style="width: {model.progress}%"></div>
            </div>
            <p class="text-sm text-gray-600 mt-2">{model.description}</p>
          </Card>
        {/each}
      </div>
    </Card>
  </div>
</div>
