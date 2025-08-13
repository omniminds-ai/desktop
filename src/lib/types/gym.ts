import type { Component, ComponentProps } from 'svelte';
import type {SubmissionStatus, Token} from './forge';
import type { TrainingPoolStatus } from './forge';

export interface ForgeTask {
  _id: string;
  prompt: string;
  uploadLimit: number;
  rewardLimit: number;
  uploadLimitReached: boolean;
  currentSubmissions: number;
  limitReason: string;
  app: {
    _id: string;
    name: string;
    domain: string;
    description: string;
    categories: string[];
    pool: {
      uploadLimit: {
        type: number;
        limitType: string;
      };
      _id: string;
      name: string;
      status: TrainingPoolStatus;
      pricePerDemo: number;
      token: Token;
      ownerAddress: string;
    };
    gymLimitType: string;
    gymSubmissions: number;
    gymLimitValue: number;
  };
}

export interface MonitorInfo {
  width: number;
  height: number;
}

export interface LocalRecording {
  id: string;
  timestamp: string;
  duration_seconds: number;
  status: string;
  title: string;
  description: string;
  platform: string;
  arch: string;
  version: string;
  locale: string;
  primary_monitor: MonitorInfo;
  quest?: Quest & { reward?: { time: number; max_reward: number } };
  submission?: SubmissionStatus;
  location?: 'local' | 'database';
}

export interface ApiRecording {
  id: string;
  timestamp: string;
  duration_seconds: number;
  status: string;
  title: string;
  description: string;
  platform: string;
  arch: string;
  version: string;
  locale: string;
  primary_monitor: MonitorInfo;
  meta?: {
    quest: Quest;
  };
  location?: 'local' | 'database';
  submission?: SubmissionStatus;
}
export interface ForgeApp {
  name: string;
  domain: string;
  description: string;
  categories: string[];
  tasks: {
    _id?: string; // Add _id field for task identification
    prompt: string;
    uploadLimit?: number;
    rewardLimit?: number;
    completed?: boolean;
    recordingId?: string;
    score?: number;
    uploadLimitReached?: boolean;
    currentSubmissions?: number;
    limitReason?: string | null;
  }[];
  pool_id: {
    _id: string;
    name: string;
    status: string;
    pricePerDemo: number;
  };
  seen?: boolean;
  gymLimitReached?: boolean;
  gymSubmissions?: number;
  gymLimitType?: 'per-task' | 'per-day' | 'total';
  gymLimitValue?: number;
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
  objectives: string[]; // List of 4 objectives, first one must be opening/navigating to the app
  content: string;
  pool_id?: string; // Match Rust struct field name
  reward?: {
    time: number; // Unix timestamp rounded to last minute
    max_reward: number; // Match Rust struct field name
    symbol: string;
  };
  task_id?: string; // ID of the specific task
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

export enum RecordingState {
  off = 'off',
  starting = 'starting',
  recording = 'recording',
  saving = 'saving',
  saved = 'saved'
}

export interface Screen {
  id: string,
  capture: string,
  name: string,
  x: number,
  y: number,
  height: number,
  width: number
}
