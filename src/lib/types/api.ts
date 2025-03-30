// Common API filter types
export interface TaskFilter {
  poolId?: string;
  minReward?: number;
  maxReward?: number;
  categories?: string[];
  query?: string;
}

// API response types can be added here if needed

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: ErrorCode;
    message: string;
    details?: Record<string, any>;
  };
}

export enum ErrorCode {
  // Client errors (4xx)
  BAD_REQUEST = 'BAD_REQUEST',
  UNAUTHORIZED = 'UNAUTHORIZED',
  INVALID_WALLET_SIGNATURE = 'INVALID_WALLET_SIGNATURE',
  FORBIDDEN = 'FORBIDDEN',
  NOT_FOUND = 'NOT_FOUND',
  REQ_VALIDATION_ERROR = 'REQ_VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  CONFLICT = 'CONFLICT',
  PAYMENT_REQUIRED = 'PAYMENT_REQUIRED',
  GONE = 'GONE',

  // Server errors (5xx)
  INTERNAL_SERVER_ERROR = 'INTERNAL_SERVER_ERROR',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',

  // Domain-specific errors
  INSUFFICIENT_FUNDS = 'INSUFFICIENT_FUNDS',
  TRANSACTION_FAILED = 'TRANSACTION_FAILED',
  CHALLENGE_EXPIRED = 'CHALLENGE_EXPIRED',
  UPLOAD_INCOMPLETE = 'UPLOAD_INCOMPLETE'
}
