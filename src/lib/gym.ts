import { invoke } from "@tauri-apps/api/core";

export async function start(prompt: string, address: string) {
  try {
    // Start recording
    await invoke('start_recording');

    // Get screenshot
    const screenshot = await invoke('take_screenshot');
    
    // Call quest endpoint
    const response = await fetch('http://localhost/api/gym/quest', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        screenshot,
        address,
        prompt,
      }),
    });

    if (!response.ok) {
      throw new Error('Failed to start quest');
    }

    return await response.json();
  } catch (error) {
    console.error('Failed to start race:', error);
    // Clean up recording on error
    await invoke('stop_recording');
    throw error;
  }
}

export async function stop() {
  try {
    await invoke('stop_recording');
  } catch (error) {
    console.error('Failed to stop race:', error);
    throw error;
  }
}
