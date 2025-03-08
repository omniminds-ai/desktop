<script lang="ts">
  import { slide } from 'svelte/transition';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import Input from '$lib/components/Input.svelte';
  import TextArea from '$lib/components/TextArea.svelte';
  import CopyButton from '$lib/components/CopyButton.svelte';
  import { onMount } from 'svelte';
  import { walletAddress } from '$lib/stores/wallet';
  import { DollarSign, Settings, RefreshCw, Upload, ListTodo } from 'lucide-svelte';

  // Placeholder data - would come from API in real implementation
  let gymName = "My Training Gym";
  let depositAddress = "HW7D5MyYG4Dz2C98axfjVBeLWpsEnofrqy6ZUwqwpump";
  let pricePerDemo = 7;
  let skills = "Web navigation\nData input\nDesktop operations\nUI interactions";
  let gymFunds = 1000;
  let activeTab = "tasks";
  let isStatusLive = true;
  let unsavedChanges = false;
  
  // Mock data for tasks tab
  let tasks = [
    { id: 1, title: "Navigate to Gmail and compose an email", category: "Web navigation", reward: 7 },
    { id: 2, title: "Create a spreadsheet with formatting", category: "Data input", reward: 8 },
    { id: 3, title: "Install a desktop application", category: "Desktop operations", reward: 10 },
    { id: 4, title: "Complete a multi-step form", category: "UI interactions", reward: 6 }
  ];
  
  // Mock data for uploads tab
  let uploads = [
    { id: 101, date: "2025-03-05", status: "completed", reward: 7, filename: "recording_20250305.mp4" },
    { id: 102, date: "2025-03-06", status: "pending", reward: 8, filename: "recording_20250306.mp4" },
    { id: 103, date: "2025-03-07", status: "completed", reward: 10, filename: "recording_20250307.mp4" }
  ];

  function toggleStatus() {
    isStatusLive = !isStatusLive;
  }
  
  function handleSkillsChange(event: Event) {
    skills = (event.target as HTMLTextAreaElement).value;
    unsavedChanges = true;
  }
  
  function handlePriceChange(event: Event) {
    pricePerDemo = parseFloat((event.target as HTMLInputElement).value);
    unsavedChanges = true;
  }
  
  function handleSaveChanges() {
    // Would make API call here
    unsavedChanges = false;
  }
  
  function refreshGymData() {
    // Would make API call here to refresh gym data
    console.log("Refreshing gym data");
  }
  
  function handleNameChange(event: Event) {
    gymName = (event.target as HTMLInputElement).value;
    unsavedChanges = true;
  }
  
  function setTab(tab: string) {
    activeTab = tab;
  }
  
  function formatNumber(num: number) {
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
</script>

<div class="h-full max-w-7xl mx-auto overflow-hidden p-4">
  <div class="mb-6">
    <h2 class="text-2xl font-bold">Gym Builder</h2>
    <p class="text-gray-400">
      Configure your training gym and manage tasks and uploads.
    </p>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 h-[calc(100vh-200px)]">
    <!-- Left Column: Gym Properties -->
    <div class="space-y-6 overflow-y-auto pb-4">
      <Card padding="md" className="relative shadow-md">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-bold">Gym Properties</h3>
          <div class="flex items-center gap-2">
            <div
              class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full
                {isStatusLive ? 'bg-green-500/10 text-green-500' : 'bg-gray-500/10 text-gray-500'}">
              <div
                class="w-1.5 h-1.5 rounded-full 
                  {isStatusLive ? 'bg-green-500 animate-pulse' : 'bg-gray-500'}">
              </div>
              {isStatusLive ? 'live' : 'paused'}
            </div>
          </div>
        </div>
        
        <!-- Gym Name -->
        <div class="mb-4">
          <label class="block text-sm font-medium mb-1">Gym Name</label>
          <Input 
            type="text" 
            value={gymName} 
            oninput={handleNameChange}
            placeholder="Enter gym name" />
        </div>
        
        <!-- Deposit Address -->
        <div class="mb-4">
          <div class="flex justify-between items-center mb-1">
            <label class="block text-sm font-medium">Deposit Address (VIRAL)</label>
            <Button
              class="px-2 py-1 text-xs"
              variant="secondary"
              onclick={refreshGymData}>
              <RefreshCw size={12} />
              <span>Refresh</span>
            </Button>
          </div>
          <div class="flex gap-2 items-center">
            <input
              type="text"
              class="flex-1 p-2 bg-gray-100 rounded-lg cursor-text text-sm"
              readonly
              value={depositAddress} />
            <CopyButton content={depositAddress} />
          </div>
          <p class="text-xs text-gray-500 mt-1">
            Send VIRAL tokens to this address to fund your gym.
          </p>
        </div>
        
        <!-- Price Settings -->
        <div class="mb-4">
          <label class="block text-sm font-medium mb-1">Price per Demonstration</label>
          <div class="flex items-center gap-2">
            <input
              type="number"
              min="1"
              step="0.1"
              class="flex-1 p-2 bg-white border border-gray-300 rounded-lg"
              value={pricePerDemo}
              oninput={handlePriceChange} />
            <span class="text-sm font-medium">VIRAL</span>
          </div>
          <p class="text-xs text-gray-500 mt-1">
            Minimum price: 1 VIRAL
          </p>
        </div>
        
        <!-- Pool Balance Info -->
        <div class="mb-6 p-3 bg-blue-50 rounded-lg border border-blue-100">
          <div class="flex justify-between mb-1">
            <span class="text-sm text-blue-800 font-medium">Current Balance:</span>
            <span class="text-sm font-bold text-blue-800">{formatNumber(gymFunds)} VIRAL</span>
          </div>
          <div class="flex justify-between">
            <span class="text-sm text-blue-800 font-medium">Possible Demos:</span>
            <span class="text-sm font-bold text-blue-800">{Math.floor(gymFunds / pricePerDemo)}</span>
          </div>
        </div>
        
        <!-- Skills to Train -->
        <div class="mb-6">
          <label class="block text-sm font-medium mb-1">Skills to Train</label>
          <TextArea
            class="h-32"
            placeholder="List the skills to train (one per line)..."
            value={skills}
            oninput={handleSkillsChange}>
          </TextArea>
          <p class="text-xs text-gray-500 mt-1">
            Each skill will generate tasks for demonstrations.
          </p>
        </div>
        
        <!-- Admin Buttons -->
        <div class="flex flex-col gap-2">
          {#if unsavedChanges}
            <Button
              class="w-full justify-center bg-green-500! text-white! hover:bg-green-600!"
              onclick={handleSaveChanges}>
              Save Changes
            </Button>
          {/if}
          
          <Button
            class="w-full justify-center {isStatusLive 
              ? 'bg-gray-200! text-gray-800! hover:bg-gray-300!' 
              : 'bg-green-500! text-white! hover:bg-green-600!'}"
            onclick={toggleStatus}>
            {isStatusLive ? 'Pause Gym' : 'Activate Gym'}
          </Button>
          
          <Button
            class="w-full justify-center"
            variant="secondary">
            <Settings size={14} class="mr-1" />
            Advanced Settings
          </Button>
        </div>
      </Card>
    </div>
    
    <!-- Right Column: Tabbed View -->
    <div class="col-span-2 flex flex-col overflow-hidden">
      <!-- Tab Navigation -->
      <div class="flex border-b border-gray-200 mb-4">
        <button
          class="px-4 py-2 text-sm font-medium transition-colors flex items-center gap-1
            {activeTab === 'tasks' 
              ? 'text-secondary-600 border-b-2 border-secondary-500' 
              : 'text-gray-500 hover:text-gray-700'}"
          onclick={() => setTab('tasks')}>
          <ListTodo size={16} />
          Tasks
        </button>
        <button
          class="px-4 py-2 text-sm font-medium transition-colors flex items-center gap-1
            {activeTab === 'uploads' 
              ? 'text-secondary-600 border-b-2 border-secondary-500' 
              : 'text-gray-500 hover:text-gray-700'}"
          onclick={() => setTab('uploads')}>
          <Upload size={16} />
          Uploads
        </button>
      </div>
      
      <!-- Tab Content -->
      <div class="flex-grow overflow-y-auto">
        {#if activeTab === 'tasks'}
          <!-- Tasks Tab -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {#each tasks as task}
              <Card padding="md" className="border border-gray-200">
                <div class="flex justify-between items-start mb-2">
                  <div class="text-sm font-medium text-gray-700 bg-gray-100 px-2 py-0.5 rounded">
                    {task.category}
                  </div>
                  <div class="text-sm font-bold text-secondary-600 flex items-center gap-1">
                    <DollarSign size={14} />
                    {task.reward} VIRAL
                  </div>
                </div>
                <p class="text-gray-800">{task.title}</p>
                <div class="flex justify-end mt-4">
                  <Button class="text-xs px-3 py-1" variant="secondary">Edit Task</Button>
                </div>
              </Card>
            {/each}
            
            <Card padding="md" className="border border-gray-200 border-dashed flex items-center justify-center hover:bg-gray-50 cursor-pointer">
              <div class="text-center">
                <div class="w-10 h-10 rounded-full bg-gray-100 flex items-center justify-center mx-auto mb-2">
                  <span class="text-xl font-bold text-gray-400">+</span>
                </div>
                <p class="text-gray-500 font-medium">Add New Task</p>
              </div>
            </Card>
          </div>
        {:else if activeTab === 'uploads'}
          <!-- Uploads Tab -->
          <Card padding="none" className="border border-gray-200">
            <table class="min-w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Filename</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Reward</th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                {#each uploads as upload}
                  <tr>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{upload.date}</td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{upload.filename}</td>
                    <td class="px-6 py-4 whitespace-nowrap">
                      <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                        {upload.status === 'completed' 
                          ? 'bg-green-100 text-green-800' 
                          : 'bg-yellow-100 text-yellow-800'}">
                        {upload.status}
                      </span>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{upload.reward} VIRAL</td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                      <a href="#" class="text-secondary-600 hover:text-secondary-900">View</a>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </Card>
        {/if}
      </div>
    </div>
  </div>
</div>
