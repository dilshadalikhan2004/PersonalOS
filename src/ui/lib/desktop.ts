import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Event, UnlistenFn } from '@tauri-apps/api/event';

export function isDesktopRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function invokeDesktop<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isDesktopRuntime()) {
    throw new Error('LifeOS desktop runtime is unavailable.');
  }
  return invoke<T>(command, args);
}

export async function listenDesktop<T>(
  event: string,
  handler: (event: Event<T>) => void,
): Promise<UnlistenFn | undefined> {
  if (!isDesktopRuntime()) return undefined;
  return listen<T>(event, handler);
}
