import type { Component, ComponentProps } from "svelte";

export interface Race {
  title: string;
  description: string;
  category: string;
  icon: string;
  skills: string[];
  agent_prompt: string;
}

export type MessageRole = "assistant" | "user" | "system";
export enum MessagePartType {
  text = "text",
  embed = "embed",
  quest = "quest",
  loading = "loading",
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
            type: "primary" | "warning" | "destroy";
            text: string;
            fn: () => void;
          }
        | {
            type: "component";
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
