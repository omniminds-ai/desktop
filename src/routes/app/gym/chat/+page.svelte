<script lang="ts">
  import { page } from '$app/state';
  import { onMount, onDestroy } from 'svelte';
  import * as gym from '$lib/gym';
  import type { Quest } from '$lib/types/gym';
  import { getReward, getSubmissionStatus } from '$lib/api/forge';
  import { walletAddress } from '$lib/stores/wallet';
  import { get } from 'svelte/store';
  import { Send, User, Video, Square, Upload } from 'lucide-svelte';
  import { emit } from '@tauri-apps/api/event';
  import Card from '$lib/components/Card.svelte';
  import Button from '$lib/components/Button.svelte';
  import Input from '$lib/components/Input.svelte';
  import tone from '$lib/assets/tone.wav';
  import blip from '$lib/assets/blip.wav';
  import pfp from '$lib/assets/V_pfp_v1.png';
  import QuestPanel from '$lib/components/gym/QuestPanel.svelte';
  import RecordingPanel from '$lib/components/gym/RecordingPanel.svelte';
  import QuestPopup from '$lib/components/gym/QuestPopup.svelte';
  import UploadConfirmModal from '$lib/components/UploadConfirmModal.svelte';
  import { handleUpload, saveUploadConfirmation } from '$lib/uploadManager';

  const prompt = page.url.searchParams.get('prompt') || '';
  const appParam = page.url.searchParams.get('app');
  const poolId = page.url.searchParams.get('poolId');
  let app: { type: 'executable' | 'website'; name: string; path?: string; url?: string } | null =
    null;

  try {
    if (appParam) {
      app = JSON.parse(decodeURIComponent(appParam));
    }
  } catch (error) {
    console.error('Failed to parse app parameter:', error);
  }

  // Quest state
  let showQuestPopup = $state(false);
  let currentQuest = $state<Quest | null>(null);
  let activeQuest = $state<Quest | null>(null);
  let recordingState = $state<'recording' | 'stopping' | 'stopped'>('stopped');
  let recordingLoading = $state(false);
  let showUploadConfirmModal = $state(false);
  let currentRecordingId = $state<string | null>(null);

  type Message = {
    role: 'user' | 'assistant';
    content: string;
  };

  let message = $state('');
  let chatMessages = $state<Message[]>([]);
  let typingMessage = $state<{
    content: string;
    target: string;
    messageIndex: number;
  } | null>(null);
  let isWaitingForResponse = $state(false);
  // Initialize variables with default values
  let chatContent: HTMLElement | null = null;
  let toneAudio: HTMLAudioElement | null = null;
  let blipAudio: HTMLAudioElement | null = null;

  const TYPE_START_DELAY = 300;

  function scrollToBottom() {
    if (chatContent) {
      chatContent.scrollTop = chatContent.scrollHeight;
    }
  }

  async function typeMessage(content: string, messageIndex: number) {
    if (!content) return;
    await new Promise((resolve) => setTimeout(resolve, TYPE_START_DELAY));

    const baseDelay = 1000 / 15 / 2; // 33.33ms per char
    const chunkSize = 2;

    for (let i = 0; i <= content.length; i += chunkSize) {
      typingMessage = {
        content: content.slice(0, i),
        target: content,
        messageIndex
      };

      const variation = baseDelay * (0.8 + Math.random() * 0.4);

      if (i % 4 === 0 || content[i - 1] === ' ') {
        if (toneAudio) {
          toneAudio.currentTime = 0;
          toneAudio.play().catch(() => {});
        }
      }

      scrollToBottom();
      await new Promise((resolve) => setTimeout(resolve, variation));
    }
    typingMessage = null;
  }

  async function addMessage(msg: Message) {
    const messageIndex = chatMessages.length;

    if (msg.role === 'assistant') {
      // Add empty message first
      chatMessages = [
        ...chatMessages,
        {
          ...msg,
          content: ''
        }
      ];
      scrollToBottom();

      // Then animate typing
      await typeMessage(msg.content, messageIndex);

      // Finally update with full message
      chatMessages = chatMessages.map((m, i) => (i === messageIndex ? msg : m));
    } else {
      if (blipAudio) {
        blipAudio.currentTime = 0;
        blipAudio.play().catch(() => {});
      }
      chatMessages = [...chatMessages, msg];
      scrollToBottom();
    }
  }

  async function sendMessage(event: Event) {
    event.preventDefault();
    if (!message.trim()) return;

    const userMessage = {
      role: 'user' as const,
      content: message
    };

    await addMessage(userMessage);
    const currentMessage = message;
    message = '';

    isWaitingForResponse = true;
    try {
      const response = await fetch('http://localhost/api/forge/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          messages: [...chatMessages],
          task_prompt: prompt || 'anything keep it simple',
          app
        })
      });
      isWaitingForResponse = false;

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const assistantResponse = await response.json();
      await handleAssistantResponse(assistantResponse);
    } catch (error) {
      isWaitingForResponse = false;
      console.error('Failed to send message:', error);
      await addMessage({
        role: 'assistant',
        content: "I'm sorry, I encountered an error processing your message. Please try again."
      });
    }
  }

  // Helper functions
  function generateAssistantMessage(content: string): Message {
    return {
      role: 'assistant',
      content
    };
  }

  async function handleQuestData(questData: Record<string, any>) {
    try {
      // If there's content, show it as a message
      if (questData?.content) {
        await addMessage(generateAssistantMessage(questData.content));
      }

      // If there's valid quest data, show the popup
      if (
        questData?.title &&
        questData?.app &&
        questData?.icon_url &&
        Array.isArray(questData?.objectives) &&
        questData.objectives.length > 0
      ) {
        currentQuest = {
          title: questData.title,
          app: questData.app,
          icon_url: questData.icon_url,
          objectives: questData.objectives,
          content: questData.content || ''
        };

        // If poolId is provided, get reward info
        if (poolId) {
          try {
            const rewardInfo = await getReward(poolId, get(walletAddress) || '');
            currentQuest.pool_id = poolId;
            currentQuest.reward = {
              time: rewardInfo.time,
              max_reward: rewardInfo.maxReward
            };
          } catch (error) {
            console.error('Failed to get reward info:', error);
          }
        }
        showQuestPopup = true;
      }
    } catch (error) {
      console.error('Failed to handle quest data:', error);
      await addMessage(
        generateAssistantMessage(
          'I encountered an error processing the quest data. Please try again.'
        )
      );
    }
  }

  async function handleToolCall(toolCall: Record<string, any>) {
    try {
      switch (toolCall?.function?.name) {
        case 'generate_quest':
        case 'validate_task_request': {
          const questData = JSON.parse(toolCall.function.arguments);
          await handleQuestData(questData);
          break;
        }
        // Add more tool call handlers here as needed
        default: {
          console.warn('Unknown tool call:', toolCall?.function?.name);
          break;
        }
      }
    } catch (error) {
      console.error('Failed to handle tool call:', error);
      await addMessage(
        generateAssistantMessage(
          'I encountered an error processing the tool call. Please try again.'
        )
      );
    }
  }

  async function toggleRecording() {
    try {
      recordingLoading = true;
      if (recordingState === 'stopped') {
        await gym.startRecording(activeQuest!).catch(console.error);
        await emit('quest-overlay', { quest: activeQuest! });
        recordingState = 'recording';
      } else if (recordingState === 'recording') {
        // await gym.stopRecording().catch(console.error);
        // recordingState = 'stopped';
        // //todo: log how many minutes/events & viralm collected
        // await addMessage({
        //   role: 'assistant',
        //   content: 'Training session ended.'
        // });
        // // Clear quest from overlay
        // await emit('quest-overlay', { quest: null });
      }
      recordingLoading = false;
    } catch (error) {
      console.error('Recording error:', error);
      recordingLoading = false;
    }
  }

  async function removeMessage() {
    // Remove the last message from the chatMessages array
    if (chatMessages.length > 0) {
      chatMessages = chatMessages.slice(0, -1);
      scrollToBottom();
    }
  }

  async function handleComplete() {
    if (recordingState === 'recording') {
      activeQuest = null;
      recordingLoading = true;
      await addMessage({
        role: 'user',
        content: `<loading></loading>`
      });
      const recordingId = await gym.stopRecording('done').catch((error) => {
        console.error(error);
        return null;
      });
      await emit('quest-overlay', { quest: null });
      recordingLoading = false;
      recordingState = 'stopped';

      await removeMessage();
      await addMessage({
        role: 'user',
        content: `<recording>${recordingId}</recording>`
      });
      await addMessage({
        role: 'user',
        content: 'I completed the task! 🎉'
      });
      await addMessage({
        role: 'assistant',
        content: 'Great job completing the task!'
      });

      // Store the recording ID for upload
      currentRecordingId = recordingId;

      await addMessage({
        role: 'assistant',
        content:
          'You can upload your demonstration to get scored now, or upload it later from the history page.'
      });

      // Add upload button message
      await addMessage({
        role: 'assistant',
        content: `<upload-button></upload-button>`
      });
    }
  }

  // Function to handle upload button click
  async function handleUploadClick() {
    if (currentRecordingId) {
      const uploadStarted = await handleUpload(currentRecordingId, activeQuest?.title || 'Unknown');
      if (!uploadStarted) {
        // If upload didn't start, show confirmation modal
        showUploadConfirmModal = true;
      } else {
        // Upload started successfully
        await addMessage({
          role: 'assistant',
          content:
            'Your demonstration is being uploaded and processed. This may take a few minutes.'
        });

        // Start polling for submission status
        pollSubmissionStatus(currentRecordingId);
      }
    }
  }

  // Handle confirmation modal actions
  function handleConfirmUpload() {
    showUploadConfirmModal = false;
    saveUploadConfirmation(true);
    if (currentRecordingId) {
      handleUpload(currentRecordingId, activeQuest?.title || 'Unknown');

      // Add message about upload
      addMessage({
        role: 'assistant',
        content: 'Your demonstration is being uploaded and processed. This may take a few minutes.'
      });

      // Start polling for submission status
      pollSubmissionStatus(currentRecordingId);
    }
  }

  function handleCancelUpload() {
    showUploadConfirmModal = false;
  }

  // Poll submission status
  let statusInterval: number | undefined = undefined;
  function pollSubmissionStatus(recordingId: string) {
    if (statusInterval) {
      clearInterval(statusInterval);
    }

    statusInterval = setInterval(async () => {
      try {
        const status = await getSubmissionStatus(recordingId);

        if (status.status === 'completed') {
          if (statusInterval) {
            clearInterval(statusInterval);
            statusInterval = undefined;
          }

          // Add message with results
          await addMessage({
            role: 'assistant',
            content: `Your demonstration has been processed! You scored ${status.clampedScore}% on this task.`
          });

          if (status.grade_result) {
            await addMessage({
              role: 'assistant',
              content: `Feedback: ${status.grade_result.summary}`
            });
          }
        } else if (status.status === 'failed') {
          if (statusInterval) {
            clearInterval(statusInterval);
            statusInterval = undefined;
          }

          // Add error message
          await addMessage({
            role: 'assistant',
            content: `There was an error processing your demonstration: ${status.error || 'Unknown error'}`
          });
        }
      } catch (error) {
        console.error('Failed to get submission status:', error);
      }
    }, 5000);
  }

  // Clean up interval on unmount
  onDestroy(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
      statusInterval = undefined;
    }
  });

  async function handleGiveUp() {
    if (recordingState === 'recording') {
      activeQuest = null;
      const recordingId = await gym.stopRecording('fail').catch((error) => {
        console.error(error);
        return null;
      });
      recordingState = 'stopped';
      await emit('quest-overlay', { quest: null });

      await addMessage({
        role: 'user',
        content: `<recording>${recordingId}</recording>`
      });
      await addMessage({
        role: 'user',
        content: 'I give up on this task.'
      });
      await addMessage({
        role: 'assistant',
        content:
          "That's okay! Not every task is for everyone. Let me know if you'd like to try something else."
      });
    }
  }

  async function handleAssistantResponse(response: Record<string, any>) {
    try {
      const toolCall = response?.tool_calls?.[0];
      if (toolCall) {
        await handleToolCall(toolCall);
      } else if (response?.role === 'assistant' && typeof response?.content === 'string') {
        await addMessage({
          role: response.role,
          content: response.content
        });
      }
    } catch (error) {
      console.error('Failed to handle assistant response:', error);
      await addMessage(
        generateAssistantMessage(
          'I encountered an error processing the response. Please try again.'
        )
      );
    }
  }

  async function getInitialMessage() {
    isWaitingForResponse = true;
    try {
      const response = await fetch('http://localhost/api/forge/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          messages: [],
          task_prompt: prompt || 'anything keep it simple',
          app
        })
      });
      isWaitingForResponse = false;

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const assistantResponse = await response.json();
      await handleAssistantResponse(assistantResponse);
    } catch (error) {
      isWaitingForResponse = false;
      console.error('Failed to get initial message:', error);
      await addMessage({
        role: 'assistant',
        content: "I'm sorry, I encountered an error starting our conversation. Please try again."
      });
    }
  }

  onMount(async () => {
    try {
      // Initialize audio instances
      toneAudio = new Audio(tone);
      toneAudio.volume = 0.15;
      blipAudio = new Audio(blip);
      blipAudio.volume = 0.15;

      // Get initial message
      await getInitialMessage();
    } catch (error) {
      console.error('Failed to initialize:', error);
    }
  });

  onDestroy(() => {
    try {
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
    } catch (error) {
      console.error('Failed to cleanup:', error);
    }
  });
</script>

<div class="h-[calc(100vh-8rem)] flex flex-col pb-[65px] relative">
  {#if showQuestPopup && currentQuest}
    <QuestPopup
      title={currentQuest.title}
      objectives={currentQuest.objectives}
      reward={currentQuest.reward ? { maxReward: currentQuest.reward.max_reward } : undefined}
      onAccept={async () => {
        showQuestPopup = false;
        // Send user's acceptance message
        await addMessage({
          role: 'user',
          content: 'Sure!'
        });
        // Set active quest
        activeQuest = currentQuest;
      }}
      onDecline={async () => {
        showQuestPopup = false;
        // Send user's decline message
        await addMessage({
          role: 'user',
          content: 'No'
        });
        // Send assistant's response
        await addMessage(
          generateAssistantMessage(
            "No problem! Let me know if you'd like to try a different quest."
          )
        );
      }} />
  {/if}
  <div
    bind:this={chatContent}
    class="flex-1 max-w-7xl w-full mx-auto px-6 pb-6 space-y-3 overflow-y-auto chat-content">
    {#each chatMessages as msg, i}
      <div
        class="flex gap-2 transition-all duration-200 {msg.role === 'user' ? 'justify-end' : ''}">
        {#if msg.role === 'user'}
          {#if msg.content.startsWith('<recording>') && msg.content.endsWith('</recording>')}
            <RecordingPanel recordingId={msg.content.slice(11, -12)} />
          {:else if msg.content.startsWith('<loading>') && msg.content.endsWith('</loading>')}
            <Card
              variant="primary"
              padding="sm"
              className="w-auto! max-w-2xl shadow-sm bg-secondary-300 text-white">
              <div
                class="h-5 w-5 rounded-full border-2 border-white border-t-transparent! animate-spin">
              </div>
            </Card>
          {:else}
            <Card
              variant="primary"
              padding="sm"
              className="w-auto! max-w-2xl shadow-sm bg-secondary-300 text-white">
              <div class="whitespace-pre-wrap tracking-wide font-medium">
                {msg.content}
              </div>
            </Card>
          {/if}
          <div
            class="shrink-0 w-8 h-8 rounded-full bg-secondary-100 text-white flex items-center justify-center shadow-md">
            <User size={18} />
          </div>
        {:else}
          <div class="shrink-0 w-8 h-8 rounded bg-secondary-300 overflow-hidden shadow-md">
            <img src={pfp} alt="V" class="w-full h-full object-cover" />
          </div>
          {#if msg.content.startsWith('<recording>') && msg.content.endsWith('</recording>')}
            <RecordingPanel recordingId={msg.content.slice(11, -12)} />
          {:else if msg.content === '<upload-button></upload-button>'}
            <Card
              variant="secondary"
              padding="sm"
              className="w-auto! shadow-sm space-y-4 relative bg-black/5">
              <div class="flex flex-col items-center gap-2">
                <Button
                  variant="primary"
                  onclick={handleUploadClick}
                  class="flex! items-center gap-1">
                  <Upload size={16} />
                  Upload Demonstration
                </Button>
                <p class="text-sm text-gray-500">Get scored and earn $VIRAL</p>
              </div>
            </Card>
          {:else}
            <Card
              variant="secondary"
              padding="sm"
              className="w-auto! shadow-sm space-y-4 relative bg-black/5">
              <div class="whitespace-pre-wrap break-words">
                {#if typingMessage && typingMessage.messageIndex === i}
                  {typingMessage.content}
                  <span class="animate-pulse">▋</span>
                {:else}
                  {msg.content}
                {/if}
              </div>
            </Card>
          {/if}
        {/if}
      </div>
    {/each}
    {#if activeQuest}
      <QuestPanel
        title={activeQuest.title}
        objectives={activeQuest.objectives}
        onStartRecording={toggleRecording}
        onComplete={handleComplete}
        onGiveUp={handleGiveUp}
        {recordingState} />
    {/if}

    {#if isWaitingForResponse}
      <div class="flex gap-2">
        <div class="shrink-0 w-8 h-8 rounded bg-secondary-300 overflow-hidden shadow-md">
          <img src={pfp} alt="V" class="w-full h-full object-cover" />
        </div>
        <Card
          variant="secondary"
          padding="sm"
          className="shadow-sm w-auto! mr-auto space-y-4 relative bg-black/5">
          <div class="flex gap-0.5 px-1.5 py-1">
            <span
              class="w-1.5 h-1.5 bg-secondary-300 rounded-full animate-[pulse_1s_ease-in-out_-0.3s_infinite]">
            </span>
            <span
              class="w-1.5 h-1.5 bg-secondary-300 rounded-full animate-[pulse_1s_ease-in-out_-0.15s_infinite]">
            </span>
            <span
              class="w-1.5 h-1.5 bg-secondary-300 rounded-full animate-[pulse_1s_ease-in-out_0s_infinite]">
            </span>
          </div>
        </Card>
      </div>
    {/if}
  </div>
</div>
<div class="p-4 absolute bottom-0 w-full left-0 bg-white border-t border-gray-200">
  <div class="flex gap-3 items-center max-w-4xl mx-auto">
    {#if recordingState === 'recording' || recordingLoading}
      <Button
        onclick={toggleRecording}
        class="bg-red-500! hover:text-red-500! hover:bg-white! border-red-500! rounded-full! flex!">
        {#if recordingState === 'recording'}
          <Square size={20} />
        {:else if recordingLoading}
          <div
            class="w-5 h-5 rounded-full border-2 border-white hover:border-red-500! border-t-transparent! animate-spin">
          </div>
        {:else}
          <Video size={20} />
        {/if}
      </Button>
    {/if}
    <form class="flex-1 flex gap-3 items-center" onsubmit={sendMessage}>
      <Input
        type="text"
        variant="light"
        disabled
        placeholder="Agent chat coming soon..."
        bind:value={message}
        class="flex-1 rounded-full!" />
      <Button type="submit" disabled class="rounded-full!">
        <Send size={20} />
      </Button>
    </form>
  </div>
</div>

<UploadConfirmModal
  open={showUploadConfirmModal}
  onConfirm={handleConfirmUpload}
  onCancel={handleCancelUpload} />
