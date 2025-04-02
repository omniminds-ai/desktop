// Common API filter types
export interface TaskFilter {
  poolId?: string;
  minReward?: number;
  maxReward?: number;
  categories?: string[];
  query?: string;
  hideAdult?: boolean;
}

// API response types can be added here if needed
