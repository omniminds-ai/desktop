export type TokenType = 'SOL' | 'VIRAL' | 'CUSTOM';

export interface Token {
  type: TokenType;
  symbol: string;
  address: string;
}

export interface TrainingPool {
  _id: string;
  name: string;
  status: 'live' | 'paused' | 'out of funds';
  demonstrations: number;
  funds: number;
  token: Token;
  skills: string;
  ownerAddress: string;
  depositAddress: string;
  expanded?: boolean;
}

export interface CreatePoolInput {
  name: string;
  skills: string;
  token: Token;
  ownerAddress: string;
}

export interface UpdatePoolInput {
  _id: string;
  status?: 'live' | 'paused';
  skills?: string;
}
