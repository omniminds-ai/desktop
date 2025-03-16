export type TokenType = 'SOL' | 'VIRAL' | 'CUSTOM';

export interface Token {
  type: TokenType;
  symbol: string;
  address: string;
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
  apps?: any[]; // Add apps array to the update input
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
  quest: {
    title: string;
    app: string;
    icon_url: string;
    objectives: string[];
    content: string;
    pool_id: string;
    reward: {
      time: number;
      max_reward: number;
    };
  };
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
