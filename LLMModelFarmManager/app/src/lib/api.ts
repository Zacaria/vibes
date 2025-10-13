import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { GameSpeed, GameState, SaveSlotSummary, Gpu } from './types';

export async function fetchState(): Promise<GameState> {
  return invoke<GameState>('get_state');
}

export async function changeSpeed(speed: GameSpeed): Promise<GameState> {
  return invoke<GameState>('set_speed', { speed });
}

export async function togglePause(paused: boolean): Promise<GameState> {
  return invoke<GameState>('set_paused', { paused });
}

export async function saveSlot(slotId: number, name: string): Promise<void> {
  await invoke('save_game', { slot_id: slotId, name });
}

export async function loadSlot(slotId: number): Promise<GameState> {
  return invoke<GameState>('load_game', { slot_id: slotId });
}

export async function listSaves(): Promise<SaveSlotSummary[]> {
  const rows = await invoke<any[]>('list_saves');
  return rows.map((row) => ({
    id: Number(row.id),
    name: String(row.name),
    created_at: String(row.created_at)
  }));
}

export async function recordMetric(key: string, value: number): Promise<void> {
  await invoke('record_metric', { key, value });
}

export async function emitTutorial(): Promise<void> {
  await invoke('emit_tutorial');
}

export function onTick(callback: (state: GameState) => void) {
  return listen<GameState>('game://tick', (event) => callback(event.payload));
}

export async function purchaseGpu(rackId: string, template: Gpu): Promise<GameState> {
  return invoke<GameState>('purchase_gpu', { rack_id: rackId, template });
}
