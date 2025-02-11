<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen, emit } from '@tauri-apps/api/event';
  import { Send, User, Keyboard, Mouse, Gamepad, Video, Square } from 'lucide-svelte';
  import GymHeader from './GymHeader.svelte';
  import Card from '../Card.svelte';
  import RecordButton from './RecordButton.svelte';
  import * as gym from '../../gym';
  import { onMount, onDestroy } from 'svelte';

  import pfp from '$lib/assets/V_pfp_v1.png';
  import tone from '$lib/assets/tone.wav';
  import blip from '$lib/assets/blip.wav';
  import { MessagePartType, type Message, type QuestInfo } from '$lib/types/gym';
  import PrivacyPolicyEmbed from './PrivacyPolicyEmbed.svelte';

  import { page } from '$app/state';
  import Button from '../Button.svelte';
  import Input from '../Input.svelte';
  import { privacyAccepted as privacyStore } from '$lib/stores/privacy';

  const { prompt = '' } = $props<{ prompt?: string }>();
  const previewSkills = page.url.searchParams.get('preview');

  let message = $state('');
  let inputEvents = $state<any[]>([]);
  let recordingState = $state<'recording' | 'stopping' | 'stopped'>('stopped');
  let isScrolled = $state(false);
  let chatMessages = $state<Message[]>([]);
  let typingMessage = $state<{
    content: string;
    target: string;
    messageIndex: number;
  } | null>(null);
  let chatContent: HTMLElement;
  let toneAudio: HTMLAudioElement;
  let blipAudio: HTMLAudioElement;
  let privacyAccepted = $state($privacyStore);
  let currentQuest = $state<QuestInfo | null>(null);
  let appIcons = $state<Record<string, string>>({});

  async function fetchAppIcons(apps: string[]) {
    const icons: Record<string, string> = {};
    for (const app of apps) {
      const response = await invoke('list_apps', { includeIcons: true });
      const appList = response as { name: string; icon?: string }[];
      const matchingApp = appList.find((a) => a.name === app);
      if (matchingApp?.icon) {
        icons[app] = matchingApp.icon;
      }
    }
    appIcons = icons;
  }

  const welcomeMessages: Message[] = [
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content: 'welcome to my gym, trainer'
        }
      ]
    },
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content: "i'm building ai agents by learning from humans like you"
        }
      ]
    },
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content:
            "i'll give you random tasks. ordering food, making spreadsheets, whatever you humans do"
        }
      ]
    },
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content: 'complete the objective exactly as i say = instant $viral'
        }
      ]
    },
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content: 'and together we show ai how humans really get things done'
        }
      ]
    },
    {
      role: 'assistant' as const,
      parts: [
        {
          type: MessagePartType.text as const,
          content: 'before we start, the privacy policy. read it.'
        }
      ]
    },
    {
      role: 'system' as const,
      parts: [
        {
          type: MessagePartType.embed as const,
          content: PrivacyPolicyEmbed,
          props: {
            privacyAccepted: () => privacyAccepted
          },
          actions: [
            {
              type: 'primary',
              text: 'Looks good to me!',
              fn: () => handlePrivacyAccept()
            },
            {
              type: 'destroy',
              text: 'Hell No!',
              fn: () => handlePrivacyDecline()
            }
          ]
        }
      ]
    }
  ];

  const privacyMessages: Message[] = welcomeMessages.slice(0, -2);

  const MAX_EVENTS = 10;
  const MESSAGE_DELAY = 500; // Shorter delay between messages
  const TYPE_START_DELAY = 300; // Small delay before typing starts

  function scrollToBottom() {
    if (chatContent) {
      chatContent.scrollTop = chatContent.scrollHeight;
    }
  }

  async function typeMessage(content: string, messageIndex: number) {
    // Add a small delay before typing starts
    await new Promise((resolve) => setTimeout(resolve, TYPE_START_DELAY));

    // Type at 15 tokens per second where each token is 2 chars
    const baseDelay = 1000 / 15 / 2; // 33.33ms per char
    const chunkSize = 2; // Type 2 chars at a time to match token size

    for (let i = 0; i <= content.length; i += chunkSize) {
      typingMessage = {
        content: content.slice(0, i),
        target: content,
        messageIndex
      };

      // Add random variation to typing speed (±20%)
      const variation = baseDelay * (0.8 + Math.random() * 0.4);

      // Play sound on word boundaries or every 4 chars
      if (i % 4 === 0 || content[i - 1] === ' ') {
        // Reset audio to start
        toneAudio.currentTime = 0;
        toneAudio.play().catch(() => {}); // Ignore errors from rapid audio plays
      }

      scrollToBottom();
      await new Promise((resolve) => setTimeout(resolve, variation));
    }
    typingMessage = null;
  }

  async function addMessage(msg: Message) {
    const messageIndex = chatMessages.length;

    if (msg.role === 'assistant' && msg.parts[0].type === 'text') {
      // Add empty message first
      chatMessages = [
        ...chatMessages,
        {
          ...msg,
          parts: [{ ...msg.parts[0], content: '' }]
        }
      ];
      scrollToBottom();

      // Then animate typing
      await typeMessage(msg.parts[0].content, messageIndex);

      // Finally update with full message
      chatMessages = chatMessages.map((m, i) => (i === messageIndex ? msg : m));
    } else {
      // Play blip sound for non-text messages
      if (msg.parts[0].type !== 'text') {
        blipAudio.currentTime = 0;
        blipAudio.play().catch(() => {});
      }
      chatMessages = [...chatMessages, msg];
      scrollToBottom();
    }
  }

  async function addMessagesWithDelay(messages: Message[]) {
    for (const msg of messages) {
      await addMessage(msg);
      if (msg.role === 'assistant') {
        await new Promise((resolve) => setTimeout(resolve, MESSAGE_DELAY));
      }
    }
  }

  onMount(() => {
    // Initialize audio instances
    toneAudio = new Audio(tone);
    toneAudio.volume = 0.15;
    blipAudio = new Audio(blip);
    blipAudio.volume = 0.15;

    const content = document.querySelector('.chat-content');
    if (content) {
      content.addEventListener('scroll', () => {
        isScrolled = content.scrollTop > 10;
      });
    }

    // Listen for recording status changes
    let unlisten: () => void;
    listen('recording-status', (event: any) => {
      recordingState = event.payload.state;
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    });

    // If in preview mode or privacy already accepted, skip welcome and go straight to quest generation
    async function init() {
      if (previewSkills) {
        privacyAccepted = true; // Skip privacy policy in preview mode
        generateQuestFromSkills(previewSkills);
      } else if (privacyAccepted) {
        // Skip welcome for returning users, just handle prompt and generate quest
        if (prompt) {
          await addMessage({
            role: 'user',
            parts: [{ type: MessagePartType.text, content: prompt }]
          });
        }
        generateQuestFromSkills(prompt || 'anything keep it simple');
      } else {
        // Start welcome messages with privacy policy for first time users
        addMessagesWithDelay(welcomeMessages);
      }
    }
    init();

    return () => {
      unlisten?.();
    };
  });

  onDestroy(() => {
    // Clean up audio instances
    if (toneAudio) {
      toneAudio.pause();
      toneAudio.src = '';
      toneAudio.remove();
    }
    if (blipAudio) {
      blipAudio.pause();
      blipAudio.src = '';
      blipAudio.remove();
    }
  });

  async function generateQuestFromSkills(skills: string) {
    await addMessage({
      role: 'assistant',
      parts: [{ type: MessagePartType.loading, content: 'Generating quest...' }]
    });

    try {
      const quest = await gym.generateQuest(skills, 'test-address');

      // Remove loading message
      chatMessages = chatMessages.slice(0, -1);

      // Store quest for recording start
      currentQuest = quest;

      // Fetch icons for relevant applications
      if (quest.relevant_applications.length > 0) {
        await fetchAppIcons(quest.relevant_applications);
      }

      await addMessage({
        role: 'assistant',
        parts: [
          {
            type: MessagePartType.quest,
            content: '',
            quest,
            actions: [
              {
                type: 'component',
                component: RecordButton,
                props: {
                  onStart: handleRecordingStart,
                  onCancel: handleRecordingCancel
                }
              },
              {
                type: 'destroy',
                text: 'Not Interested',
                fn: () => handleQuestDecline()
              }
            ]
          }
        ]
      });
    } catch (error) {
      console.error('Failed to generate quest:', error);
      // Remove loading message
      chatMessages = chatMessages.slice(0, -1);

      await addMessage({
        role: 'assistant',
        parts: [
          {
            type: MessagePartType.text,
            content: 'sorry, something went wrong generating your quest. try again later.'
          }
        ]
      });
    }
  }

  async function handlePrivacyAccept() {
    privacyAccepted = true;
    privacyStore.set(true);
    await addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: 'Looks good to me!' }]
    });
    await addMessage({
      role: 'assistant',
      parts: [{ type: MessagePartType.text, content: 'welcome aboard.' }]
    });
    await addMessage({
      role: 'assistant',
      parts: [
        {
          type: MessagePartType.text,
          content: "let's create some agents that'll change everything."
        }
      ]
    });
    if (prompt) {
      await addMessage({
        role: 'user',
        parts: [{ type: MessagePartType.text, content: prompt }]
      });
      await addMessage({
        role: 'assistant',
        parts: [
          {
            type: MessagePartType.text,
            content: "i'll help you get started with some tasks that'll help train our AI and earn you rewards."
          }
        ]
      });
    }
    generateQuestFromSkills(prompt || 'anything keep it simple');
  }

  function handlePrivacyDecline() {
    addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: 'Hell no!' }]
    });
    addMessage({
      role: 'assistant',
      parts: [
        {
          type: MessagePartType.text,
          content: "ok. come back when you're ready."
        }
      ]
    });
  }

  async function handleRecordingStart() {
    addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: "Let's do this! 💪" }]
    });
    // Emit quest-overlay event when recording actually starts
    if (currentQuest) {
      await emit('quest-overlay', { quest: currentQuest });
    }
    toggleRecording();
  }

  function handleRecordingCancel() {
    addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: 'Recording cancelled' }]
    });
  }

  function handleQuestDecline() {
    addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: 'Not interested.' }]
    });
  }

  async function toggleRecording() {
    try {
      if (recordingState === 'stopped') {
        gym.startRecording().catch(console.error);
      } else if (recordingState === 'recording') {
        gym.stopRecording().catch(console.error);
        // Clear quest from overlay
        await emit('quest-overlay', { quest: null });
      }
    } catch (error) {
      console.error('Recording error:', error);
    }
  }

  async function sendMessage(event: Event) {
    event.preventDefault();
    if (!message.trim()) return;

    await addMessage({
      role: 'user',
      parts: [{ type: MessagePartType.text, content: message }]
    });
    const currentMessage = message;
    message = '';

    try {
      if (currentMessage.toLowerCase() === 'list apps') {
        const apps = await invoke('list_apps', { includeIcons: true });
        console.log('Installed apps:', apps);
        console.log(
          (apps as { name: any; path: any }[])
            .map((app: { name: any; path: any }) => `${app.name}`)
            .join('\n')
        );
        await addMessage({
          role: 'assistant',
          parts: [
            {
              type: MessagePartType.text,
              content: "I've logged the list of installed apps to the console."
            }
          ]
        });
      } else {
        const response = await invoke('handle_message', {
          message: currentMessage
        });
        await addMessage({
          role: 'assistant',
          parts: [{ type: MessagePartType.text, content: response as string }]
        });
      }
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  }

  function formatEvent(event: any): string {
    const { type, data } = event;
    switch (type) {
      case 'KeyPress':
        return `Key pressed: ${data.key}`;
      case 'KeyRelease':
        return `Key released: ${data.key}`;
      case 'MouseMove':
        return `Mouse position: (${data.x.toFixed(1)}, ${data.y.toFixed(1)})`;
      case 'MouseDelta':
        return `Mouse delta: (${data.x}, ${data.y})`;
      case 'MouseClick':
        return `Mouse ${data.state.toLowerCase()}: ${data.button}`;
      case 'MouseWheel':
        return `Mouse wheel: ${data.delta}`;
      case 'JoystickButton':
        return `Joystick ${data.id} button ${data.button}: ${data.state.toLowerCase()}`;
      case 'JoystickAxis':
        return `Joystick ${data.id} axis ${data.axis}: ${data.value.toFixed(2)}`;
      default:
        return JSON.stringify(event);
    }
  }

  // Listen for global input events from Rust
  listen<any>('input-event', (event: { payload: any }) => {
    inputEvents = [event.payload, ...inputEvents.slice(0, MAX_EVENTS - 1)];
  });
</script>

<div class="h-full flex flex-col relative">
  <div class="pt-4 px-4">
    <GymHeader title="Training Session" />
  </div>
  <div
    bind:this={chatContent}
    class="flex-1 px-6 pb-6 space-y-3 overflow-y-auto chat-content">
    {#each chatMessages as msg, i}
      <div
        class="flex gap-2 transition-all duration-200 {msg.role === 'user' ? 'justify-end' : ''}">
        {#if msg.role === 'user'}
          <Card variant="primary" className="max-w-2xl shadow-sm bg-secondary-300 text-white">
            {#each msg.parts as part}
              <div class="whitespace-pre-wrap tracking-wide font-medium">
                {part.content}
              </div>
            {/each}
          </Card>
          <div
            class="shrink-0 w-8 h-8 rounded-full bg-secondary-100 text-white flex items-center justify-center shadow-md">
            <User size={18} />
          </div>
        {:else}
          <div class="shrink-0 w-8 h-8 rounded bg-secondary-300 overflow-hidden shadow-md">
            <img src={pfp} alt="V" class="w-full h-full object-cover" />
          </div>
          <Card variant="secondary" className="max-w-2xl shadow-sm space-y-4 relative bg-black/5">
            {#each msg.parts as part}
              <div>
                {#if part.type === MessagePartType.embed}
                  <part.content {...part.props}></part.content>
                {:else if part.type === MessagePartType.loading}
                  <div class="flex items-center gap-2">
                    <div
                      class="w-4 h-4 rounded-full border-2 border-secondary-300 border-t-transparent animate-spin">
                    </div>
                    <span class="text-sm">{part.content}</span>
                  </div>
                {:else if part.type === MessagePartType.quest && part.quest}
                  <div
                    class="p-4 rounded bg-black/10 space-y-4 hover:bg-black/15 transition-colors duration-200 border border-secondary-300/20">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <span
                          class="px-2 py-1 text-xs font-semibold bg-secondary-300 text-white rounded-full">
                          New Quest
                        </span>
                        <h3 class="text-sm tracking-tight uppercase">
                          {part.quest.title}
                        </h3>
                      </div>
                      <div class="px-2 py-1 text-xs font-semibold bg-secondary-300/10 rounded-full">
                        Task #{part.quest.task_id}
                      </div>
                    </div>
                    <p class="text-base">{part.quest.concrete_scenario}</p>
                    <div class="text-sm font-medium">
                      Objective: {part.quest.objective}
                    </div>
                    <div class="space-y-2">
                      {#each part.quest.subgoals as subgoal}
                        <div class="flex items-center gap-2 text-sm opacity-75">
                          <div class="w-1.5 h-1.5 rounded-full bg-secondary-300"></div>
                          <span>{subgoal}</span>
                        </div>
                      {/each}
                    </div>
                    {#if part.quest.relevant_applications.length > 0}
                      <div class="flex flex-wrap gap-2 mt-2">
                        {#each part.quest.relevant_applications as app}
                          <div
                            class="flex items-center gap-1 px-2 py-1 text-xs bg-secondary-300/5 rounded-full">
                            {#if appIcons[app]}
                              <img src={appIcons[app]} alt={app} class="w-4 h-4 object-contain" />
                            {/if}
                            <span>{app}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {:else}
                  <div class="whitespace-pre-wrap">
                    {#if typingMessage && typingMessage.messageIndex === i}
                      {typingMessage.content}
                      <span class="animate-pulse">▋</span>
                    {:else}
                      {part.content}
                    {/if}
                  </div>
                {/if}
                {#if part.actions}
                  {#each part.actions as action}
                    <div class="flex gap-2 mt-4">
                      {#if action.type === 'primary'}
                        <Button onclick={action.fn}>
                          {action.text}
                        </Button>
                      {:else if action.type === 'destroy'}
                        <Button variant="destroy" onclick={action.fn}>
                          {action.text}
                        </Button>
                      {:else if action.type === 'warning'}
                        <Button variant="warning" onclick={action.fn}>
                          {action.text}
                        </Button>
                      {:else if action.type === 'component'}
                        <action.component {...action.props}></action.component>
                      {/if}
                    </div>
                  {/each}
                {/if}
              </div>
            {/each}
          </Card>
        {/if}
      </div>
    {/each}

    {#if inputEvents.length > 0}
      <div class="flex gap-4">
        <div
          class="shrink-0 w-8 h-8 rounded-full bg-secondary-300 text-white flex items-center justify-center shadow-sm">
          {#if inputEvents.length > 0 && inputEvents[0].type.startsWith('Key')}
            <Keyboard size={18} />
          {:else if inputEvents.length > 0 && inputEvents[0].type.startsWith('Joystick')}
            <Gamepad size={18} />
          {:else}
            <Mouse size={18} />
          {/if}
        </div>
        <Card variant="secondary" className="max-w-2xl shadow-sm">
          <div class="space-y-2">
            {#each inputEvents as event}
              <div class="text-sm font-mono">{formatEvent(event)}</div>
            {/each}
          </div>
        </Card>
      </div>
    {/if}
  </div>
  <div>
    <div class="p-4 bg-white border-t border-gray-200">
      <div class="flex gap-3 items-center max-w-4xl mx-auto">
        <Button
          onclick={toggleRecording}
          class={(recordingState === 'recording'
            ? 'bg-red-500! hover:text-red-500! hover:bg-white! border-red-500!'
            : '') + ' rounded-full! flex!'}>
          {#if recordingState === 'recording'}
            <Square size={20} />
          {:else}
            <Video size={20} />
          {/if}
        </Button>
        <form class="flex-1 flex gap-3 items-center" onsubmit={sendMessage}>
          <Input
            type="text"
            variant="light"
            placeholder="Type your message..."
            bind:value={message}
            class="flex-1 rounded-full!" />
          <Button type="submit" class="rounded-full!">
            <Send size={20} />
          </Button>
        </form>
      </div>
    </div>
  </div>
</div>
