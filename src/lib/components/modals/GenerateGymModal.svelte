<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Button from '$lib/components/form/Button.svelte';
  import TextArea from '$lib/components/form/TextArea.svelte';
  import { ArrowLeft, Sparkles, RefreshCw } from 'lucide-svelte';
  import { generateApps } from '$lib/api/endpoints/forge';
  import type { GenerateResponse } from '$lib/types/forge';

  // Props
  export let show = false;
  export let skills = '';

  // Local state
  let currentStep = 1; // Step 1: Input skills, Step 2: Generating, Step 3: Preview
  let selectedPrompt = '';
  let generating = false;
  let generatedResponse: GenerateResponse | null = null;
  let rawAppsJson = '';
  let error: string | null = null;
  let showJsonEditor = false;

  const dispatch = createEventDispatcher();

  // Example prompts for different computer-use agent tasks
  const examplePrompts = [
    {
      label: 'Video Editing',
      text: 'Video editing with tools like Premiere Pro, After Effects, and DaVinci Resolve'
    },
    {
      label: 'Gaming',
      text: 'Playing and streaming Steam games, managing game libraries, and configuring game settings'
    },
    {
      label: 'Food Ordering',
      text: 'Ordering food on platforms like DoorDash, UberEats, and Grubhub'
    },
    {
      label: 'Spreadsheets',
      text: 'Creating and managing spreadsheets in Excel, Google Sheets, or other tools'
    },
    {
      label: 'Documents',
      text: 'Writing and editing documents in Word, Google Docs, or other word processors'
    },
    {
      label: 'Coding',
      text: 'Using VSCode or other IDEs for programming, debugging, and version control'
    },
    {
      label: '3D Modeling',
      text: 'Creating 3D models and animations in Blender, Maya, or other modeling software'
    },
    { label: 'Social Media', text: 'Engaging on Twitter, Instagram, and other social platforms' },
    {
      label: 'Community Management',
      text: 'Setting up and managing Discord servers, Telegram groups, or Slack workspaces'
    },
    {
      label: 'Crypto Trading',
      text: 'Trading cryptocurrency on exchanges, managing wallets, and analyzing market trends'
    }
  ];

  function handleSkillsChange(event: Event) {
    skills = (event.target as HTMLTextAreaElement).value;
    dispatch('change', { skills });
  }

  function applyExamplePrompt(prompt: string) {
    skills = prompt;
    dispatch('change', { skills });
  }

  function handleCancel() {
    currentStep = 1;
    generatedResponse = null;
    showJsonEditor = false;
    dispatch('close');
  }

  async function startGeneration() {
    if (!skills.trim()) return;

    currentStep = 2; // Move to generating step
    generating = true;
    error = null;

    try {
      generatedResponse = await generateApps(skills);
      if (generatedResponse && generatedResponse.content) {
        updateRawJson();
        currentStep = 3; // Move to preview step
      } else {
        error = 'Failed to generate content';
        currentStep = 1;
      }
    } catch (err) {
      console.error('Failed to generate:', err);
      error = err instanceof Error ? err.message : 'Failed to generate apps';
      currentStep = 1;
    } finally {
      generating = false;
    }
  }

  function updateRawJson() {
    if (generatedResponse && generatedResponse.content && generatedResponse.content.apps) {
      rawAppsJson = JSON.stringify(generatedResponse.content.apps, null, 2);
    }
  }

  function updateJsonFromEditor() {
    try {
      if (generatedResponse && generatedResponse.content) {
        generatedResponse.content.apps = JSON.parse(rawAppsJson);
        return true;
      }
      return false;
    } catch (error: any) {
      alert('Invalid JSON: ' + error.message);
      return false;
    }
  }

  function toggleJsonEditor() {
    // If switching from JSON to UI, try to update the content first
    if (showJsonEditor) {
      const success = updateJsonFromEditor();
      if (!success) return; // Don't switch if JSON is invalid
    } else {
      // Going from UI to JSON, update the JSON text
      updateRawJson();
    }
    showJsonEditor = !showJsonEditor;
  }

  function updateGymName(event: Event) {
    const value = (event.target as HTMLElement).textContent || '';
    if (generatedResponse && generatedResponse.content) {
      generatedResponse.content.name = value;
    }
  }

  function updateAppName(app: any, event: Event) {
    const value = (event.target as HTMLElement).textContent || '';
    app.name = value;
  }

  function updateTaskPrompt(task: any, event: Event) {
    const value = (event.target as HTMLElement).textContent || '';
    task.prompt = value;
  }

  function goBackToEdit() {
    currentStep = 1;
  }

  function handleSave() {
    currentStep = 1;
    dispatch('save', { generatedResponse });
  }
</script>

{#if show}
  <div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
    <div class="bg-white rounded-lg w-full max-w-2xl">
      <div class="p-4 border-b border-gray-200 flex justify-between items-center">
        <h3 class="text-lg font-bold">Generate a gym</h3>
        <button class="text-gray-500 hover:text-gray-700" onclick={handleCancel}>&times;</button>
      </div>

      {#if currentStep === 1}
        <!-- Step 1: Input skills -->
        <div class="p-4">
          <p class="text-sm text-gray-600 mb-3">
            You can generate a gym by describing your dream agent, or by sharing the skills you want
            a training dataset for.
          </p>
          <TextArea
            class="h-48"
            variant="light"
            placeholder="List the skills to train (one per line)..."
            value={skills}
            oninput={handleSkillsChange}>
          </TextArea>

          <!-- Example prompt buttons -->
          <div class="mt-3">
            <p class="text-sm text-gray-600 mb-2">Example prompts:</p>
            <div class="flex flex-wrap gap-2">
              {#each examplePrompts as prompt}
                <button
                  class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-800 py-1 px-2 rounded-full transition-colors"
                  onclick={() => applyExamplePrompt(prompt.text)}>
                  {prompt.label}
                </button>
              {/each}
            </div>
          </div>

          {#if error}
            <div class="mt-3 p-3 bg-red-50 text-red-500 rounded-lg text-sm">
              {error}
            </div>
          {/if}
        </div>
      {:else if currentStep === 2}
        <!-- Step 2: Generating -->
        <div class="p-4 flex flex-col items-center justify-center" style="min-height: 300px;">
          <div class="mb-6">
            <RefreshCw size={48} class="text-secondary-300 animate-spin" />
          </div>
          <p class="text-lg font-medium text-gray-800">Generating your gym</p>
          <p class="text-sm text-gray-500 mt-2">Creating tasks based on your requirements...</p>
        </div>
      {:else if currentStep === 3}
        <!-- Step 3: Preview -->
        <div class="p-4">
          <div class="flex justify-between mb-2">
            <div>
              <h4 class="text-xl font-bold">Your gym</h4>
              <h5 class="text-sm text-gray-600">You'll be able to make further changes later</h5>
            </div>
          </div>

          {#if showJsonEditor}
            <!-- JSON Editor -->
            <div class="my-4">
              <TextArea
                class="h-[350px] font-mono text-sm"
                variant="light"
                placeholder="JSON for apps..."
                bind:value={rawAppsJson}>
              </TextArea>
              <div class="flex justify-end mt-2">
                <!-- Removed explicit save button since changes apply automatically -->
              </div>
            </div>
          {:else}
            <!-- UI Preview -->
            {#if generatedResponse && generatedResponse.content}
              <div class="bg-gray-50 p-4 rounded-lg mt-4">
                <div
                  class="text-lg font-bold text-gray-800 mb-3 border border-transparent rounded px-2 py-1 hover:border-gray-200 cursor-text"
                  contenteditable="true"
                  onblur={updateGymName}>
                  {generatedResponse.content.name || 'Desktop Agent Gym'}
                </div>

                {#if generatedResponse.content.apps && generatedResponse.content.apps.length > 0}
                  <div class="mt-4 space-y-4 max-h-[350px] overflow-y-auto pr-2">
                    {#each generatedResponse.content.apps as app}
                      <div class="bg-white rounded-lg p-3 shadow-sm border border-gray-200">
                        <div class="flex items-center gap-2 mb-2">
                          <div class="w-6 h-6 flex items-center justify-center">
                            <img
                              src={`https://www.google.com/s2/favicons?domain=${app.domain || app.name.toLowerCase().replace(/\s+/g, '') + '.com'}&sz=32`}
                              alt={app.name}
                              class="w-5 h-5"
                              onerror={(e) => {
                                (e.currentTarget as HTMLImageElement).src =
                                  'data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%2224%22 height=%2224%22 viewBox=%220 0 24 24%22 fill=%22none%22 stroke=%22currentColor%22 stroke-width=%222%22 stroke-linecap=%22round%22 stroke-linejoin=%22round%22><rect width=%2218%22 height=%2218%22 x=%223%22 y=%223%22 rx=%222%22 ry=%222%22></rect><path d=%22M9 17V9l7 4-7 4Z%22></path></svg>';
                              }} />
                          </div>
                          <div
                            class="font-medium border border-transparent rounded px-2 py-0.5 hover:border-gray-200 cursor-text"
                            contenteditable="true"
                            onblur={(e) => updateAppName(app, e)}>
                            {app.name}
                          </div>
                        </div>

                        {#if app.tasks && app.tasks.length > 0}
                          <div class="ml-8 space-y-2">
                            {#each app.tasks as task}
                              <div
                                class="text-sm text-gray-700 bg-gray-50 p-2 rounded-md border border-transparent hover:border-gray-300 cursor-text"
                                contenteditable="true"
                                onblur={(e) => updateTaskPrompt(task, e)}>
                                {task.prompt}
                              </div>
                            {/each}
                          </div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {:else}
                  <div class="text-center py-4 text-gray-500">No apps or tasks generated</div>
                {/if}
              </div>
            {/if}
          {/if}
        </div>
      {/if}

      <div class="p-4 border-t border-gray-200 flex justify-between">
        {#if currentStep === 1}
          <Button variant="secondary" onclick={handleCancel}>Cancel</Button>
          <Button
            disabled={!skills.trim() || generating}
            onclick={startGeneration}
            class="flex! items-center">
            <span class="flex items-center">
              <Sparkles size={16} class="mr-1" />
              Generate
            </span>
          </Button>
        {:else if currentStep === 2}
          <!-- No buttons during generation -->
          <div></div>
          <div></div>
        {:else if currentStep === 3}
          <Button variant="secondary" onclick={goBackToEdit}>
            <span class="flex items-center">
              <ArrowLeft size={16} class="mr-1" />
              Back
            </span>
          </Button>
          <Button onclick={handleSave}>Continue</Button>
        {/if}
      </div>
    </div>
  </div>
{/if}
