import { apiClient } from '$lib/api';

/**
 * Get gym leaderboard data
 * @returns Promise resolving to the leaderboard data
 */
export async function getLeaderboardData(): Promise<{
  workersLeaderboard: {
    rank: number;
    address: string;
    nickname?: string;
    tasks: number;
    rewards: number;
  }[];
  forgeLeaderboard: {
    rank: number;
    name: string;
    tasks: number;
    payout: number;
  }[];
  stats: {
    totalWorkers: number;
    tasksCompleted: number;
    totalRewards: number;
    activeForges: number;
  };
}> {
  return apiClient.get<{
    workersLeaderboard: {
      rank: number;
      address: string;
      nickname?: string;
      tasks: number;
      rewards: number;
    }[];
    forgeLeaderboard: {
      rank: number;
      name: string;
      tasks: number;
      payout: number;
    }[];
    stats: {
      totalWorkers: number;
      tasksCompleted: number;
      totalRewards: number;
      activeForges: number;
    };
  }>('/gym/leaderboards');
}
