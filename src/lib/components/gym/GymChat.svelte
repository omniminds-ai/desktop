<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import { Command } from '@tauri-apps/plugin-shell';
import { platform } from '@tauri-apps/plugin-os';
import { Send, User, Keyboard, Mouse, Gamepad, Video, Square, ArrowLeft } from 'lucide-svelte';
import Card from '../Card.svelte';
import * as gym from '../../gym';
import { onMount, onDestroy } from 'svelte';
import PrivacyPolicy from "../PrivacyPolicy.svelte";

import pfp from "$lib/assets/V_pfp_v1.png";
import tone from "$lib/assets/tone.wav";
import blip from "$lib/assets/blip.wav";

const { prompt = "" } = $props<{ prompt?: string }>();

type MessageRole = 'assistant' | 'user' | 'system';
type MessagePartType = 'text' | 'privacy' | 'quest';

interface QuestInfo {
  title: string;
  description: string;
  reward: number;
  objectives: Array<{
    text: string;
    completed: boolean;
  }>;
}

interface MessagePart {
  type: MessagePartType;
  content: string;
  quest?: QuestInfo;
  actions?: {
    accept?: () => void;
    decline?: () => void;
  };
}

interface Message {
  role: MessageRole;
  parts: MessagePart[];
}

let message = $state("");
let inputEvents = $state<any[]>([]);
let recordingState = $state<'recording' | 'stopping' | 'stopped'>('stopped');
let isScrolled = $state(false);
let chatMessages = $state<Message[]>([]);
let typingMessage = $state<{content: string, target: string, messageIndex: number} | null>(null);
let chatContent: HTMLElement;
let toneAudio: HTMLAudioElement;
let blipAudio: HTMLAudioElement;
let privacyAccepted = $state(false);

const welcomeMessages = [
  
{
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "welcome to my gym, trainer" }]
  },
  {
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "i build ai agents, and i need your data" }]
  },
  {
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "i'll give you random tasks. ordering food, making spreadsheets, whatever you humans do" }]
  },
  {
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "complete the objective exactly as i say = instant $viral" }]
  },
  {
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "and together we show ai how humans really get things done" }]
  },
  {
    role: 'assistant' as const,
    parts: [{ type: 'text' as const, content: "before we start, the privacy policy. read it." }]
  },
  {
    role: 'system' as const,
    parts: [{
      type: 'privacy' as const,
      content: '',
      actions: {
        accept: () => handlePrivacyAccept(),
        decline: () => handlePrivacyDecline()
      }
    }]
  }
];

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
  await new Promise(resolve => setTimeout(resolve, TYPE_START_DELAY));
  
  // Type at 15 tokens per second where each token is 2 chars
  const baseDelay = (1000 / 15) / 2; // 33.33ms per char
  const chunkSize = 2; // Type 2 chars at a time to match token size
  
  for (let i = 0; i <= content.length; i += chunkSize) {
    typingMessage = { content: content.slice(0, i), target: content, messageIndex };
    
    // Add random variation to typing speed (±20%)
    const variation = baseDelay * (0.8 + Math.random() * 0.4);
    
    // Play sound on word boundaries or every 4 chars
    if (i % 4 === 0 || content[i-1] === ' ') {
      // Reset audio to start
      toneAudio.currentTime = 0;
      toneAudio.play().catch(() => {}); // Ignore errors from rapid audio plays
    }
    
    scrollToBottom();
    await new Promise(resolve => setTimeout(resolve, variation));
  }
  typingMessage = null;
}

async function addMessage(msg: Message) {
  const messageIndex = chatMessages.length;
  
  if (msg.role === 'assistant' && msg.parts[0].type === 'text') {
    // Add empty message first
    chatMessages = [...chatMessages, {
      ...msg,
      parts: [{ ...msg.parts[0], content: '' }]
    }];
    scrollToBottom();
    
    // Then animate typing
    await typeMessage(msg.parts[0].content, messageIndex);
    
    // Finally update with full message
    chatMessages = chatMessages.map((m, i) => 
      i === messageIndex ? msg : m
    );
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
      await new Promise(resolve => setTimeout(resolve, MESSAGE_DELAY));
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
  }).then(unlistenFn => {
    unlisten = unlistenFn;
  });

  // Start welcome messages
  addMessagesWithDelay(welcomeMessages);

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

async function handlePrivacyAccept() {
  privacyAccepted = true;
  await addMessage({
    role: 'user',
    parts: [{ type: 'text', content: 'Looks good to me!' }]
  });
  await addMessage({
    role: 'assistant',
    parts: [{ type: 'text', content: "welcome aboard." }]
  });
  await addMessage({
    role: 'assistant',
    parts: [{ type: 'text', content: "let's create some agents that'll change everything." }]
  });
  await addMessage({
    role: 'assistant',
    parts: [{ type: 'text', content: "your first task:" }]
  });
  await addMessage({
    role: 'assistant',
    parts: [{
      type: 'quest',
      content: '',
      quest: {
        title: 'folder renaming',
        description: 'create a new folder and rename it 3 times',
        reward: 10,
        objectives: [
          { text: "Create a new folder", completed: false },
          { text: "Rename it once", completed: false },
          { text: "Rename it twice more", completed: false }
        ]
      },
      actions: {
        accept: () => handleQuestAccept(),
        decline: () => handleQuestDecline()
      }
    }]
  });
}

function handlePrivacyDecline() {
  addMessage({
    role: 'user',
    parts: [{ type: 'text', content: 'Hell no!' }]
  });
  addMessage({
    role: 'assistant',
    parts: [{ type: 'text', content: "ok. come back when you're ready." }]
  });
}

function handleQuestAccept() {
  addMessage({
    role: 'user',
    parts: [{ type: 'text', content: "Let's do this!" }]
  });
  toggleRecording();
}

function handleQuestDecline() {
  addMessage({
    role: 'user',
    parts: [{ type: 'text', content: 'Not interested.' }]
  });
}

async function toggleRecording() {
  try {
    if (recordingState === 'stopped') {
      gym.start(prompt || "Free Race", "test-address").catch(console.error);
    } else if (recordingState === 'recording') {
      gym.stop().catch(console.error);
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
    parts: [{ type: 'text', content: message }]
  });
  const currentMessage = message;
  message = "";
  
  try {
    const response = await invoke("handle_message", { message: currentMessage });
    await addMessage({
      role: 'assistant',
      parts: [{ type: 'text', content: response as string }]
    });
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
listen<any>('input-event', (event: { payload: any; }) => {
  inputEvents = [event.payload, ...inputEvents.slice(0, MAX_EVENTS - 1)];
});
</script>

<div class="h-full flex flex-col relative">
  <a 
    href="/app/gym" 
    class="absolute z-10 top-4 left-4 px-4 py-2 flex items-center gap-2 rounded-full
      transition-all duration-200 ease-in-out text-[var(--vm-secondary-300)] border
      {isScrolled 
        ? 'bg-white/75 backdrop-blur-sm border-[var(--vm-secondary-300)] hover:bg-white shadow-md' 
        : 'bg-white/0 hover:text-[var(--vm-secondary-400)] border-transparent'
      }"
  >
    <ArrowLeft size={20} />
    Back to Gym
  </a>
  <div bind:this={chatContent} class="flex-1 pt-16 px-6 pb-6 space-y-3 overflow-y-auto chat-content">
    {#each chatMessages as msg, i}
      <div class="flex gap-2 transition-all duration-200 {msg.role === 'user' ? 'justify-end' : ''}">
        {#if msg.role === 'user'}
          <Card variant="primary" className="max-w-2xl shadow-sm bg-[var(--vm-secondary-300)] text-white">
            {#each msg.parts as part}
              <div class="whitespace-pre-wrap tracking-wide font-medium">{part.content}</div>
            {/each}
          </Card>
          <div class="shrink-0 w-8 h-8 rounded-full bg-[var(--vm-secondary-100)] text-white flex items-center justify-center shadow-md">
            <User size={18} />
          </div>
        {:else}
          <div class="shrink-0 w-8 h-8 rounded bg-[var(--vm-secondary-300)] overflow-hidden shadow-md">
            <img src={pfp} alt="V" class="w-full h-full object-cover" />
          </div>
          <Card variant="secondary" className="max-w-2xl shadow-sm space-y-4 relative bg-black/5">
            {#each msg.parts as part}
              <div>
                {#if part.type === 'privacy'}
                  {#if privacyAccepted}
                    <a 
                      href="https://github.com/viralmind-ai/gym-desktop/blob/main/PRIVACY.md" 
                      class="text-[var(--vm-secondary-300)] hover:underline"
                      target="_blank"
                    >
                      View Privacy Policy
                    </a>
                  {:else}
                    <PrivacyPolicy />
                  {/if}
                {:else if part.type === 'quest' && part.quest}
                  <div class="p-4 rounded bg-black/10 space-y-4 hover:bg-black/15 transition-colors duration-200 border border-[var(--vm-secondary-300)]/20">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <span class="px-2 py-1 text-xs font-semibold bg-[var(--vm-secondary-300)] text-white rounded-full">New Quest</span>
                        <h3 class="text-sm tracking-tight uppercase">{part.quest.title}</h3>
                      </div>
                      <div class="px-2 py-1 text-xs font-semibold bg-[var(--vm-secondary-300)]/10 rounded-full">
                        {part.quest.reward} $viral
                      </div>
                    </div>
                    <p class="text-base">{part.quest.description}</p>
                    <div class="space-y-2">
                      {#each part.quest.objectives as objective}
                        <div class="flex items-center gap-2 text-sm opacity-75">
                          <div class="w-1.5 h-1.5 rounded-full bg-[var(--vm-secondary-300)]"></div>
                          <span>{objective.text}</span>
                        </div>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="whitespace-pre-wrap">
                    {#if typingMessage && typingMessage.messageIndex === i}
                      {typingMessage.content}<span class="animate-pulse">▋</span>
                    {:else}
                      {part.content}
                    {/if}
                  </div>
                {/if}
                {#if part.actions}
                  <div class="flex gap-2 mt-4">
                    {#if part.actions.accept}
                      <button 
                        onclick={part.actions.accept}
                        class="px-4 py-2 bg-[var(--vm-secondary-300)] text-white rounded hover:bg-[var(--vm-secondary-400)] hover:shadow-md transition-all duration-200 tracking-wide font-medium"
                      >
                        {part.type === 'privacy' ? 'Looks good to me!' : "Let's do this!"}
                      </button>
                    {/if}
                    {#if part.actions.decline}
                      <button 
                        onclick={part.actions.decline}
                        class="px-4 py-2 border border-[var(--vm-secondary-300)] text-[var(--vm-secondary-300)] rounded hover:bg-[var(--vm-secondary-300)] hover:text-white hover:shadow-md transition-all duration-200 tracking-wide font-medium"
                      >
                        {part.type === 'privacy' ? 'Hell no!' : 'Not interested.'}
                      </button>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </Card>
        {/if}
      </div>
    {/each}

    {#if inputEvents.length > 0}
      <div class="flex gap-4">
        <div class="shrink-0 w-8 h-8 rounded-full bg-[var(--vm-secondary-300)] text-white flex items-center justify-center shadow-sm">
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
        <button 
          onclick={toggleRecording}
          class="p-3 rounded-full {recordingState === 'recording' ? 'bg-red-500' : 'bg-[var(--vm-secondary-300)]'} text-white hover:opacity-90 transition-colors shadow-sm hover:shadow-md"
        >
          {#if recordingState === 'recording'}
            <Square size={20} />
          {:else}
            <Video size={20} />
          {/if}
        </button>
        <form class="flex-1 flex gap-3 items-center" onsubmit={sendMessage}>
          <input 
            type="text" 
            placeholder="Type your message..."
            bind:value={message}
            class="flex-1 px-6 py-3 rounded-full border border-gray-200 bg-gray-50 focus:outline-none focus:ring-2 focus:ring-[var(--vm-secondary-300)] focus:bg-white transition-colors"
          />
          <button 
            type="submit" 
            class="p-3 rounded-full bg-[var(--vm-secondary-300)] text-white hover:bg-[var(--vm-secondary-400)] transition-colors shadow-sm hover:shadow-md"
          >
            <Send size={20} />
          </button>
        </form>
      </div>
    </div>
  </div>
</div>
