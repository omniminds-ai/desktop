import { error } from '@sveltejs/kit';

export const load = async ({ url }) => {
  // Get the gymId from the querystring instead of route params
  const gymId = url.searchParams.get('id');
  
  if (!gymId) {
    throw error(404, 'Gym ID is required');
  }
  
  return {
    gymId
  };
};
