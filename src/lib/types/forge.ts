import type { ForgeApp, Quest } from '$lib/types/gym';

export type TokenType = 'SOL' | 'VIRAL' | 'CUSTOM';

export interface Token {
  type: TokenType;
  symbol: string;
  address: string;
}

export interface RewardInfo {
  time: number; // Unix timestamp rounded to last minute
  maxReward: number; // Value between 1-128
}
export interface GenerateResponse {
  content: {
    name: string;
    apps: ForgeApp[];
  };
}

// Update create pool interface to accept apps
export interface CreatePoolInputWithApps extends CreatePoolInput {
  apps?: ForgeApp[];
}

export interface PoolSubmission {
  _id: string;
  address: string;
  meta: {
    poolId: string;
    id: string;
    generatedTime: number;
    quest: {
      pool_id: string;
      app_id: string;
      title?: string;
      app?: string;
    };
    [key: string]: any;
  };
  status: string; // PENDING, PROCESSING, COMPLETED, FAILED, etc.
  files: {
    file: string;
    s3Key: string;
    _id?: string;
    size?: number;
  }[];
  grade_result?: any;
  reward: number;
  maxReward: number;
  clampedScore: number;
  createdAt: string;
  updatedAt: string;
}

export enum TrainingPoolStatus {
  live = 'live',
  paused = 'paused',
  noFunds = 'no-funds',
  noGas = 'no-gas'
}

export interface TrainingPool {
  _id: string;
  name: string;
  status: TrainingPoolStatus;
  demonstrations: number;
  funds: number;
  token: Token;
  skills: string;
  ownerAddress: string;
  depositAddress: string;
  expanded?: boolean;
  unsavedSkills?: boolean;
  pricePerDemo?: number;
  unsavedPrice?: boolean;
  uploadLimit?: {
    type: number;
    limitType: 'per-task' | 'per-day' | 'total';
  } | null;
  unsavedUploadLimit?: boolean;
}

export interface CreatePoolInput {
  name: string;
  skills: string;
  token: Token;
  ownerAddress: string;
}

export interface UpdatePoolInput {
  id: string;
  status?: TrainingPoolStatus.live | TrainingPoolStatus.paused;
  skills?: string;
  pricePerDemo?: number;
  uploadLimit?: {
    type: number;
    limitType: 'per-task' | 'per-day' | 'total';
  } | null;
  apps?: {
    name: string;
    domain: string;
    description: string;
    categories: string[];
    tasks: {
      _id?: string;
      prompt: string;
      uploadLimit?: number;
      rewardLimit?: number;
    }[];
    pool_id: {
      _id: string;
      name: string;
      status: string;
      pricePerDemo: number;
    };
  }[]; // Properly typed apps array with task limits
}

export interface TreasuryTransfer {
  tokenAddress: string;
  treasuryWallet: string;
  amount: number;
  timestamp: number;
  txHash: string;
  _id: string;
}

export interface GradeResult {
  summary: string;
  score: number;
  reasoning: string;
  _id: string;
}

export interface SubmissionMeta {
  id: string;
  timestamp: string;
  duration_seconds: number;
  status: string;
  reason: string;
  title: string;
  description: string;
  platform: string;
  arch: string;
  version: string;
  locale: string;
  primary_monitor: {
    width: number;
    height: number;
  };
  quest: Quest;
}

export interface SubmissionStatus {
  _id: string;
  address: string;
  meta: SubmissionMeta;
  status: string;
  files: Array<{
    file: string;
    s3Key: string;
    _id: string;
  }>;
  error?: string;
  createdAt: string;
  updatedAt: string;
  clampedScore: number;
  grade_result: GradeResult;
  maxReward: number;
  reward: number;
  treasuryTransfer?: TreasuryTransfer;
}
