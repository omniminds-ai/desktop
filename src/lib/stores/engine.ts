import { invoke } from '@tauri-apps/api/core';

// Type definitions
export interface EngineStatus {
  running: boolean;
  system_info: string | null;
  gpu_info: string | null;
  error: string | null;
}

export enum EngineType {
  Native = 'Native',
  WSL = 'WSL',
  SSH = 'SSH',
}

export interface WSLConfig {
  distro: string;
}

export interface SSHConfig {
  host: string;
  port: number;
  username: string;
  password?: string;
  private_key_path?: string;
}

export interface EngineConfig {
  engine_type: EngineType;
  wsl_config?: WSLConfig;
  ssh_config?: SSHConfig;
}

// Default engine configuration
const defaultConfig: EngineConfig = {
  engine_type: EngineType.Native
};

/**
 * Start the engine with the specified configuration
 */
export async function startEngine(config: EngineConfig = defaultConfig): Promise<EngineStatus> {
  try {
    return await invoke('start_engine', { config });
  } catch (error) {
    console.error('Failed to start engine:', error);
    throw error;
  }
}

/**
 * Stop the engine
 */
export async function stopEngine(): Promise<EngineStatus> {
  try {
    return await invoke('stop_engine');
  } catch (error) {
    console.error('Failed to stop engine:', error);
    throw error;
  }
}

/**
 * Get the current engine status
 */
export async function getEngineStatus(): Promise<EngineStatus> {
  try {
    return await invoke('get_engine_status');
  } catch (error) {
    console.error('Failed to get engine status:', error);
    throw error;
  }
}

/**
 * Run a command using the engine
 */
export async function runCommand(command: string): Promise<string> {
  try {
    return await invoke('run_command', { command });
  } catch (error) {
    console.error('Failed to run command:', error);
    throw error;
  }
}
