import type { Component, ComponentProps } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';

export interface ForgeApp {
  name: string;
  domain: string;
  description: string;
  categories: string[];
  tasks: {
    prompt: string;
    completed?: boolean;
    recordingId?: string;
    score?: number;
  }[];
  pool_id: {
    _id: string;
    name: string;
    status: string;
    pricePerDemo: number;
  };
  seen?: boolean;
}

export type MessageRole = 'assistant' | 'user' | 'system';
export enum MessagePartType {
  text = 'text',
  embed = 'embed',
  quest = 'quest',
  loading = 'loading'
}

export interface Quest {
  title: string;
  app: string;
  icon_url: string;
  objectives: string[];  // List of 4 objectives, first one must be opening/navigating to the app
  content: string;
  pool_id?: string;  // Match Rust struct field name
  reward?: {
    time: number;      // Unix timestamp rounded to last minute
    max_reward: number; // Match Rust struct field name
  };
}

export interface QuestInfo {
  task_id: string;
  title: string;
  original_instruction: string;
  concrete_scenario: string;
  objective: string;
  relevant_applications: string[];
  subgoals: string[];
}

export type MessagePart<T extends Component = any, P = ComponentProps<T>> = {
  quest?: QuestInfo;
  actions?:
    | (
        | {
            type: 'primary' | 'warning' | 'destroy';
            text: string;
            props?: P;
            fn: () => void;
          }
        | {
            type: 'component';
            component: T;
            props: P;
          }
      )[];
} & (
  | {
      type: MessagePartType.embed;
      content: T;
      props: P;
    }
  | {
      type: Exclude<MessagePartType, MessagePartType.embed>;
      content: string;
    }
);

export interface Message<T extends Component = any, P = ComponentProps<T>> {
  role: MessageRole;
  parts: MessagePart<T, P>[];
}
