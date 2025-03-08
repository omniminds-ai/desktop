import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
  // The gymId is loaded dynamically through the page params
  // and will be accessible in the +page.svelte component
  const { gymId } = params;
  
  if (!gymId) {
    throw error(404, 'Gym ID is required');
  }
  
  return {
    gymId
  };
};
