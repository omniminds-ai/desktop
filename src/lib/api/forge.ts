import type { TrainingPool, CreatePoolInput, UpdatePoolInput } from '$lib/types/forge';

const API_BASE = 'http://localhost/api/forge';

export async function listPools(address: string): Promise<TrainingPool[]> {
  const response = await fetch(`${API_BASE}/list`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ address }),
  });

  if (!response.ok) {
    throw new Error('Failed to fetch training pools');
  }

  return response.json();
}

export async function createPool(input: CreatePoolInput): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/create`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(input),
  });

  if (!response.ok) {
    throw new Error('Failed to create training pool');
  }

  return response.json();
}

export async function updatePool(input: UpdatePoolInput): Promise<TrainingPool> {
  const response = await fetch(`${API_BASE}/update`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(input),
  });

  if (!response.ok) {
    throw new Error('Failed to update training pool');
  }

  return response.json();
}
