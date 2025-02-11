export type TokenType = 'SOL' | 'VIRAL' | 'CUSTOM';

export interface Token {
  type: TokenType;
  symbol: string;
  address: string;
}

export enum TrainingPoolStatus {
  live = 'live',
  paused = 'paused',
  noFunds = 'no-funds'
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
}
