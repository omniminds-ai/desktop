<script lang="ts">
  import { onMount } from 'svelte';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import { Shield, Eye, EyeOff } from 'lucide-svelte';

  let isLoading = true;
  let privacyEnabled = false;
  
  onMount(async () => {
    try {
      // In a real implementation, this would load privacy settings from backend
      privacyEnabled = false;
    } catch (error) {
      console.error('Error loading privacy settings:', error);
    } finally {
      isLoading = false;
    }
  });

  function togglePrivacy() {
    privacyEnabled = !privacyEnabled;
    // In a real implementation, this would save to backend
  }
</script>

<div class="h-full max-w-7xl mx-auto">
  {#if isLoading}
    <p>Loading privacy settings...</p>
  {:else}
    <div class="space-y-6">
      <Card padding="md" className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
        <div class="flex items-center gap-3 mb-4">
          <div class="bg-indigo-100 p-3 rounded-full">
            <Shield size={24} class="text-indigo-600" />
          </div>
          <div>
            <h2 class="text-xl font-semibold">Recording Privacy</h2>
            <p class="text-sm text-gray-500">Control privacy settings for your screen recordings</p>
          </div>
        </div>
        
        <div class="p-4 bg-gray-50 rounded-lg mb-4">
          <div class="flex justify-between items-center">
            <div class="flex items-center">
              {#if privacyEnabled}
                <Eye size={20} class="mr-2 text-green-600" />
                <span class="font-medium">Privacy Enabled</span>
              {:else}
                <EyeOff size={20} class="mr-2 text-gray-500" />
                <span class="font-medium">Privacy Disabled</span>
              {/if}
            </div>
            
            <div class="relative inline-block w-12 h-6 transition duration-200 ease-in-out rounded-full cursor-pointer">
              <input 
                type="checkbox" 
                id="toggle" 
                class="absolute w-6 h-6 opacity-0 cursor-pointer"
                bind:checked={privacyEnabled}
                on:change={togglePrivacy}
              />
              <label 
                for="toggle" 
                class={`block h-6 overflow-hidden rounded-full cursor-pointer transition-colors duration-200 ease-in-out ${privacyEnabled ? 'bg-green-500' : 'bg-gray-300'}`}
              >
                <span 
                  class={`block w-6 h-6 rounded-full transform transition-transform duration-200 ease-in-out bg-white shadow-md ${privacyEnabled ? 'translate-x-6' : 'translate-x-0'}`}
                ></span>
              </label>
            </div>
          </div>
        </div>
        
        <div class="space-y-4">
          <div class="p-3 border border-gray-200 rounded-lg">
            <h3 class="font-medium mb-2">What gets protected?</h3>
            <ul class="list-disc ml-5 text-sm text-gray-600 space-y-1">
              <li>Sensitive text in recordings</li>
              <li>Personal identifiable information</li>
              <li>Credit card numbers and financial data</li>
              <li>Passwords and authentication tokens</li>
            </ul>
          </div>
          
          <div class="p-3 border border-gray-200 rounded-lg">
            <h3 class="font-medium mb-2">How it works</h3>
            <p class="text-sm text-gray-600">
              When privacy mode is enabled, the system automatically detects and blurs sensitive information
              in your screen recordings. This helps protect your private data while still capturing the
              essential elements of your workflow.
            </p>
          </div>
        </div>
      </Card>
    </div>
  {/if}
</div>
