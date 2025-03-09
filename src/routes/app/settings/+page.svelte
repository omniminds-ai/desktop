<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { platform } from '@tauri-apps/plugin-os';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import Input from '$lib/components/Input.svelte';
  import TextArea from '$lib/components/TextArea.svelte';
  import { Save, HardDrive, Settings, Cpu, Play, Square, SendHorizonal, Terminal } from 'lucide-svelte';
  
  interface SettingsData {
    engines: {
      backend: string;
      wslDistro: string;
      sshHost: string;
      sshPort: number;
      sshUsername: string;
      sshPrivateKeyPath: string;
      downloadsPath: string;
    };
    storage: {
      recordingsPath: string;
    };
  }
  
  let settings: SettingsData = {
    engines: {
      backend: 'native',
      wslDistro: '',
      sshHost: 'localhost',
      sshPort: 22,
      sshUsername: '',
      sshPrivateKeyPath: '',
      downloadsPath: '~/.viralmind/downloads'
    },
    storage: {
      recordingsPath: ''
    }
  };
  
  interface EngineStatus {
    running: boolean;
    system_info?: string;
    gpu_info?: string;
    error?: string;
  }

  // Engine state
  let engineStatus: EngineStatus = {
    running: false
  };
  let commandInput = '';
  let commandOutput = '';
  let commandHistory: string[] = [];
  let isSending = false;
  let isStartingEngine = false;
  let isStoppingEngine = false;
  let errorMessage = '';
  let showErrorMessage = false;

  let isSaving = false;
  let hasChanges = false;
  let initialSettings: SettingsData;
  let isWindows = false;
  let wslDistros: string[] = [];

  // Function to get engine status
  async function getEngineStatus() {
    try {
      engineStatus = await invoke<EngineStatus>('get_engine_status');
    } catch (error) {
      console.error('Failed to get engine status:', error);
    }
  }

  // Function to start the engine
  async function startEngine() {
    if (engineStatus.running) return;
    
    isStartingEngine = true;
    showErrorMessage = false;
    errorMessage = '';
    
    try {
      // Build engine config (simplified for new engine.rs structure)
      const engineConfig = {
        engine_type: settings.engines.backend.toUpperCase(),
        wsl_config: settings.engines.backend === 'wsl' ? {
          distro: settings.engines.wslDistro
        } : null,
        ssh_config: settings.engines.backend === 'ssh' ? {
          host: settings.engines.sshHost,
          port: settings.engines.sshPort,
          username: settings.engines.sshUsername,
          private_key_path: settings.engines.sshPrivateKeyPath || null,
          password: null
        } : null
      };
      
      engineStatus = await invoke<EngineStatus>('start_engine', { config: engineConfig });
      commandOutput = '';
      if (engineStatus.system_info) {
        commandOutput += `System Info:\n${engineStatus.system_info}\n\n`;
      }
      if (engineStatus.gpu_info) {
        commandOutput += `GPU Info:\n${engineStatus.gpu_info}`;
      }
      
      if (engineStatus.error) {
        errorMessage = `Engine reported an error: ${engineStatus.error}`;
        showErrorMessage = true;
      }
    } catch (error) {
      console.error('Failed to start engine:', error);
      errorMessage = `Failed to start engine: ${error}`;
      showErrorMessage = true;
      commandOutput = `Error starting engine: ${error}`;
    } finally {
      isStartingEngine = false;
    }
  }

  // Function to stop the engine
  async function stopEngine() {
    if (!engineStatus.running) return;
    
    isStoppingEngine = true;
    try {
      engineStatus = await invoke<EngineStatus>('stop_engine');
      commandOutput += '\n\nEngine stopped.';
    } catch (error) {
      console.error('Failed to stop engine:', error);
      commandOutput += `\n\nError stopping engine: ${error}`;
    } finally {
      isStoppingEngine = false;
    }
  }

  // Function to send a command to the engine
  async function sendCommand() {
    if (!engineStatus.running || !commandInput.trim()) return;
    
    isSending = true;
    try {
      // Build engine config same as startEngine (simplified for new engine.rs structure)
      const engineConfig = {
        engine_type: settings.engines.backend.toUpperCase(),
        wsl_config: settings.engines.backend === 'wsl' ? {
          distro: settings.engines.wslDistro
        } : null,
        ssh_config: settings.engines.backend === 'ssh' ? {
          host: settings.engines.sshHost,
          port: settings.engines.sshPort,
          username: settings.engines.sshUsername,
          private_key_path: settings.engines.sshPrivateKeyPath || null,
          password: null
        } : null
      };
      
      const cmd = commandInput.trim();
      commandHistory.push(cmd);
      commandHistory = commandHistory; // Trigger svelte reactivity
      commandOutput += `\n\n> ${cmd}`;
      
      const jobId = await invoke<string>('run_command', { 
        command: cmd
      });
      
      // Poll for job completion
      const checkJob = async () => {
        try {
          const job = await invoke<any>('get_job', { jobId });
          if (job.status !== 'Running') {
            if (job.status === 'Success') {
              commandOutput += `\n${job.output}`;
            } else {
              commandOutput += `\nError: ${job.error}`;
            }
            return true;
          }
          return false;
        } catch (error) {
          console.error('Error checking job status:', error);
          commandOutput += `\nError checking command status: ${error}`;
          return true;
        }
      };
      
      // Check every 500ms until complete
      const poll = async () => {
        if (await checkJob()) return;
        setTimeout(poll, 500);
      };
      
      await poll();
      commandInput = '';
    } catch (error) {
      console.error('Failed to send command:', error);
      commandOutput += `\nError: ${error}`;
    } finally {
      isSending = false;
      // Auto-scroll to bottom of output
      setTimeout(() => {
        const outputElement = document.getElementById('command-output');
        if (outputElement) {
          outputElement.scrollTop = outputElement.scrollHeight;
        }
      }, 10);
    }
  }
  
  // Default recording path
  let defaultRecordingPath = '';

  async function getDefaultRecordingPath() {
    try {
      const appDataDir = await invoke<string>('get_app_data_dir');
      defaultRecordingPath = `${appDataDir}/recordings`;
    } catch (error) {
      console.error('Failed to get default recording path:', error);
      defaultRecordingPath = 'Unknown';
    }
  }

  onMount(async () => {
    // Listen for engine status updates
    const unlisten = await listen('engine-status', (event) => {
      engineStatus = event.payload as EngineStatus;
    });
    
    // Listen for job updates
    const unlistenJob = await listen('job-updated', (event) => {
      const job = event.payload as any;
      if (job.status !== 'Running') {
        if (job.status === 'Success' && job.output) {
          commandOutput += `\n${job.output}`;
        } else if (job.error) {
          commandOutput += `\nError: ${job.error}`;
        }
        
        // Auto-scroll to bottom of output
        setTimeout(() => {
          const outputElement = document.getElementById('command-output');
          if (outputElement) {
            outputElement.scrollTop = outputElement.scrollHeight;
          }
        }, 10);
      }
    });

    // Get the default recording path
    await getDefaultRecordingPath();
    
    // Get initial engine status
    await getEngineStatus();
    
    // Check if running on Windows
    try {
      const os = platform();
      isWindows = os === 'windows';
      
      // On Windows, load WSL distros and get default
      let defaultWslDistro = '';
      if (isWindows) {
        await loadWslDistros();
        try {
          defaultWslDistro = await invoke<string>('get_default_wsl_distro');
        } catch (error) {
          console.error('Failed to get default WSL distro:', error);
        }
      }
      
      // Load settings
      try {
        const loadedSettings = await invoke<any>('get_settings');
        
        // On Windows, default to WSL backend with default distro
        // On other platforms, default to native
        const defaultBackend = isWindows ? 'wsl' : 'native';
        
        settings = {
          engines: {
            backend: loadedSettings?.engines?.backend ?? defaultBackend,
            wslDistro: loadedSettings?.engines?.wsl_config?.distro ?? defaultWslDistro,
            sshHost: loadedSettings?.engines?.ssh_config?.host ?? 'localhost',
            sshPort: loadedSettings?.engines?.ssh_config?.port ?? 22,
            sshUsername: loadedSettings?.engines?.ssh_config?.username ?? '',
            sshPrivateKeyPath: loadedSettings?.engines?.ssh_config?.private_key_path ?? '',
            downloadsPath: loadedSettings?.engines?.downloads_path ?? '~/.viralmind/downloads'
          },
          storage: {
            recordingsPath: loadedSettings?.storage?.recordings_path ?? ''
          }
        };
        initialSettings = JSON.parse(JSON.stringify(settings));
      } catch (error) {
        console.error('Failed to load settings:', error);
        
        // Set defaults even if loading fails
        settings.engines.backend = isWindows ? 'wsl' : 'native';
        settings.engines.wslDistro = isWindows ? defaultWslDistro : '';
        settings.engines.sshHost = 'localhost';
        settings.engines.sshPort = 22;
        settings.engines.sshUsername = '';
        settings.engines.sshPrivateKeyPath = '';
        initialSettings = JSON.parse(JSON.stringify(settings));
      }
    } catch (error) {
      console.error('Failed to check OS:', error);
      initialSettings = JSON.parse(JSON.stringify(settings));
    }
  });
  
  async function loadWslDistros() {
    try {
      // Use invoke instead of shell command
      const result = await invoke<string>('list_wsl_distros');
      wslDistros = result.trim().split('\n').map((distro: string) => distro.trim());
    } catch (error) {
      console.error('Failed to get WSL distros:', error);
      wslDistros = [];
    }
  }
  
  function handleChange() {
    hasChanges = JSON.stringify(settings) !== JSON.stringify(initialSettings);
  }
  
  async function saveSettings() {
    if (!hasChanges) return;
    
    isSaving = true;
    try {
      // Transform the settings back to Rust format
      const rustSettings = {
        upload_confirmed: true, // Preserve existing setting
        engines: {
          backend: settings.engines.backend,
          downloads_path: settings.engines.downloadsPath,
          wsl_config: {
            distro: settings.engines.wslDistro || ''
          },
          ssh_config: {
            host: settings.engines.sshHost,
            port: settings.engines.sshPort,
            username: settings.engines.sshUsername,
            private_key_path: settings.engines.sshPrivateKeyPath || null,
            password: null // For security, don't store password
          }
        },
        storage: {
          recordings_path: settings.storage.recordingsPath
        }
      };
      
      await invoke('save_settings', { settings: rustSettings });
      initialSettings = JSON.parse(JSON.stringify(settings));
      hasChanges = false;
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      isSaving = false;
    }
  }
  
  async function selectPath(pathType: 'recordings' | 'downloads') {
    try {
      const selected = await open({
        directory: true,
        multiple: false
      });
      
      if (selected) {
        if (pathType === 'recordings') {
          settings.storage.recordingsPath = selected as string;
        } else {
          settings.engines.downloadsPath = selected as string;
        }
        handleChange();
      }
    } catch (error) {
      console.error(`Failed to select ${pathType} directory:`, error);
    }
  }
</script>

<div class="container max-w-4xl mx-auto p-4">
  <div class="mb-8 flex items-center">
    <Settings size={24} class="mr-2" />
    <h1 class="text-2xl font-bold">Settings</h1>
  </div>
  
  <div class="flex flex-col space-y-8 gap-3">

    <!-- Storage Settings -->
    <section>
      <h2 class="text-xl font-semibold mb-4 flex items-center">
        <HardDrive size={20} class="mr-2" />
        Storage
      </h2>
      <Card padding="md" className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1">Recordings Path</label>
            <div class="flex gap-2">
              <Input
                type="text"
                variant="light"
                value={settings.storage.recordingsPath}
                placeholder={defaultRecordingPath}
                readonly
                class="flex-1"
              />
              <Button
                variant="secondary"
                disabled={true}
                onclick={() => selectPath('recordings')}
              >
                Browse
              </Button>
            </div>
            <p class="text-xs text-gray-500 mt-1">
              Location where screen recordings are stored. Default: {defaultRecordingPath}
            </p>
          </div>
        </div>
      </Card>
    </section>

    <!-- Engine Control Panel -->
    <section>
      <h2 class="text-xl font-semibold mb-4 flex items-center">
        <Terminal size={20} class="mr-2" />
        Engine Control
      </h2>
      <Card padding="md" className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
        <div class="space-y-4">
          <div class="flex justify-between items-center">
            <div>
              <span class="font-medium">Engine Status:</span>
              <span class={`ml-2 ${engineStatus.running ? 'text-green-500' : 'text-red-500'}`}>
                {engineStatus.running ? 'Running' : 'Stopped'}
              </span>
            </div>
            <div class="flex gap-2">
              <Button
                variant="primary"
                class={`px-4 py-2 ${engineStatus.running ? 'bg-gray-400' : 'bg-green-600'} text-white flex! items-center`}
                onclick={startEngine}
                disabled={engineStatus.running || isStartingEngine}
              >
                {#if isStartingEngine}
                  <div class="animate-spin mr-2">⚙️</div>
                  Starting...
                {:else}
                  <Play size={16} class="mr-2" />
                  Start Engine
                {/if}
              </Button>
              
              <Button
                variant="secondary"
                class={`px-4 py-2 ${!engineStatus.running ? 'bg-gray-400' : 'bg-red-600 border-red-400! hover:bg-red-400'} text-white flex! items-center`}
                onclick={stopEngine}
                disabled={!engineStatus.running || isStoppingEngine}
              >
                {#if isStoppingEngine}
                  <div class="animate-spin mr-2">⚙️</div>
                  Stopping...
                {:else}
                  <Square size={16} class="mr-2" />
                  Stop Engine
                {/if}
              </Button>
            </div>
          </div>
          {#if showErrorMessage}
            <div class="mt-4 p-3 bg-red-100 border border-red-300 rounded-lg text-red-700">
              <p class="text-sm font-medium">{errorMessage}</p>
            </div>
          {/if}
          
          {#if engineStatus.running}
            <div class="mt-4">
              <label class="block text-sm font-medium mb-2">Command Output</label>
              <div 
                id="command-output"
                class="bg-black/90 border border-gray-500 border-4 text-green-400 p-4 rounded-lg h-64 overflow-y-auto font-mono text-sm whitespace-pre-wrap"
              >
                {commandOutput || 'No output yet.'}
              </div>
              
              <div class="flex gap-2 mt-4">
                <Input
                  variant="light"
                  type="text"
                  placeholder="Enter command..."
                  bind:value={commandInput}
                  class="flex-1"
                  onkeypress={(e) => e.key === 'Enter' && sendCommand()}
                />
                <Button
                  variant="primary"
                  class="px-4 py-2 bg-blue-600 text-white"
                  onclick={sendCommand}
                  disabled={isSending || !commandInput.trim()}
                >
                  {#if isSending}
                    <div class="animate-spin mr-2">⚙️</div>
                    Sending...
                  {:else}
                    <SendHorizonal size={16} class="mr-2" />
                    Send
                  {/if}
                </Button>
              </div>
              
              {#if commandHistory.length > 0}
                <div class="mt-2">
                  <label class="block text-sm font-medium mb-2">Command History</label>
                  <div class="flex flex-wrap gap-1">
                    {#each commandHistory.slice(-5).reverse() as cmd}
                      <button
                        class="px-2 py-1 bg-gray-200 hover:bg-gray-300 rounded text-xs"
                        on:click={() => commandInput = cmd}
                      >
                        {cmd.length > 20 ? cmd.substring(0, 20) + '...' : cmd}
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </Card>
    </section>
    
    <!-- Engine Settings -->
    <section>
      <h2 class="text-xl font-semibold mb-4 flex items-center">
        <Cpu size={20} class="mr-2" />
        Engines
      </h2>
      <Card padding="md" className="border border-gray-200 hover:shadow-md transition-shadow duration-300">
        <div class="space-y-4">
          <div>
            <h3 class="font-medium mb-1">Backend</h3>
            <div class="grid grid-cols-2 gap-3">
              <button 
                class={`p-3 rounded-lg border ${settings.engines.backend === 'native' ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}`}
                on:click={() => { settings.engines.backend = 'native'; handleChange(); }}
              >
                Native
              </button>
              <button 
                class={`p-3 rounded-lg border ${settings.engines.backend === 'ssh' ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}`}
                on:click={() => { settings.engines.backend = 'ssh'; handleChange(); }}
              >
                SSH
              </button>
              
              {#if isWindows}
                <button 
                  class={`p-3 rounded-lg border ${settings.engines.backend === 'wsl' ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}`}
                  on:click={() => { settings.engines.backend = 'wsl'; handleChange(); }}
                >
                  WSL
                </button>
              {/if}
            </div>
          </div>
          
          <div>
            <label class="block text-sm font-medium mb-1">Downloads Path</label>
            <div class="flex gap-2">
              <Input
                type="text"
                variant="light"
                value={settings.engines.downloadsPath}
                placeholder="~/.viralmind/downloads"
                readonly
                class="flex-1"
              />
            </div>
            <p class="text-xs text-gray-500 mt-1">
              Location where downloaded files are stored
            </p>
          </div>
          
          {#if isWindows && settings.engines.backend === 'wsl'}
            <div>
              <label class="block text-sm font-medium mb-1">WSL Distribution</label>
              <select 
                class="w-full p-2 bg-gray-100 rounded-lg border-0"
                bind:value={settings.engines.wslDistro}
                on:change={handleChange}
              >
                <option value="">Select a distribution</option>
                {#each wslDistros as distro}
                  <option value={distro}>{distro}</option>
                {/each}
              </select>
              {#if wslDistros.length === 0}
                <p class="text-xs text-red-500 mt-1">
                  No WSL distributions found. Please install WSL and a Linux distribution.
                </p>
              {/if}
            </div>
            
          {/if}
          
          {#if settings.engines.backend === 'ssh'}
            <div class="space-y-3">
              <div>
                <label class="block text-sm font-medium mb-1">SSH Host</label>
                <Input
                  type="text"
                  variant="light"
                  bind:value={settings.engines.sshHost}
                  oninput={handleChange}
                  class="w-full"
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium mb-1">SSH Port</label>
                <Input
                  type="number"
                  variant="light"
                  bind:value={settings.engines.sshPort}
                  oninput={handleChange}
                  class="w-full"
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium mb-1">SSH Username</label>
                <Input
                  type="text"
                  variant="light"
                  bind:value={settings.engines.sshUsername}
                  oninput={handleChange}
                  class="w-full"
                />
              </div>
              
              <div>
                <label class="block text-sm font-medium mb-1">SSH Private Key Path</label>
                <div class="flex gap-2">
                  <Input
                    type="text"
                    variant="light"
                    bind:value={settings.engines.sshPrivateKeyPath}
                    oninput={handleChange}
                    class="flex-1"
                  />
                  <Button
                    variant="secondary"
                    onclick={async () => {
                      try {
                        const selected = await open({
                          directory: false,
                          multiple: false,
                          filters: [{
                            name: 'Private Key',
                            extensions: ['pem', 'key', 'ppk']
                          }]
                        });
                        
                        if (selected) {
                          settings.engines.sshPrivateKeyPath = selected as string;
                          handleChange();
                        }
                      } catch (error) {
                        console.error('Failed to select private key file:', error);
                      }
                    }}
                  >
                    Browse
                  </Button>
                </div>
                <p class="text-xs text-gray-500 mt-1">
                  Path to your SSH private key file
                </p>
              </div>
              
              <div class="mt-4">
                <div class="flex items-center">
                  <label class="block text-sm font-medium mb-1">Advanced Settings</label>
                  <button 
                    type="button" 
                    class="ml-2 text-blue-500 text-sm underline"
                    on:click={() => {
                      const advancedSection = document.getElementById('advanced-ssh-settings');
                      if (advancedSection) {
                        advancedSection.classList.toggle('hidden');
                      }
                    }}
                  >
                    Show/Hide
                  </button>
                </div>
                
                <div id="advanced-ssh-settings" class="hidden mt-2">
                  <p class="text-xs text-gray-500 mt-1">
                    Commands will be executed using bash in the SSH environment for consistent behavior.
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </Card>
    </section>
  </div>
  
  {#if hasChanges}
    <div class="fixed bottom-4 right-4 z-10">
      <Button
        variant="primary"
        class="px-6 py-3 bg-blue-600 text-white "
        onclick={saveSettings}
        disabled={isSaving}
      >
        {#if isSaving}
          <div class="animate-spin mr-2">⚙️</div>
          Saving...
        {:else}
          <Save size={16} class="mr-2" />
          Save Changes
        {/if}
      </Button>
    </div>
  {/if}
</div>
