import { derived, get, writable } from 'svelte/store';
import {
  changeSpeed,
  emitTutorial,
  fetchState,
  listSaves,
  loadSlot,
  onTick,
  recordMetric,
  saveSlot,
  togglePause,
  purchaseGpu
} from './api';
import type { GameSpeed, GameState, SaveSlotSummary, Gpu } from './types';

const defaultState: GameState = {
  tick: 0,
  speed: 'Normal',
  paused: false,
  cash: 0,
  profit_per_hour: 0,
  tokens_served_per_hour: 0,
  uptime: 1,
  carbon_intensity: 0,
  power_draw_kw: 0,
  data_centers: [],
  models: [],
  workloads: [],
  token_providers: [],
  user_segments: [],
  ledger: [],
  events: [],
  active_incidents: []
};

const state = writable<GameState>(defaultState);
const loading = writable<boolean>(true);
const errorMessage = writable<string>('');
const saves = writable<SaveSlotSummary[]>([]);

let unsubscribeTick: (() => void) | null = null;
let autosaveTimer: ReturnType<typeof setInterval> | null = null;

export const kpiStore = derived(state, ($state) => ({
  cash: $state.cash,
  profit: $state.profit_per_hour,
  tokens: $state.tokens_served_per_hour,
  uptime: $state.uptime,
  carbon: $state.carbon_intensity,
  power: $state.power_draw_kw,
  speed: $state.speed,
  paused: $state.paused
}));

export async function initGame() {
  try {
    loading.set(true);
    const snapshot = await fetchState();
    state.set(snapshot);
    await refreshSaves();
    if (!unsubscribeTick) {
      unsubscribeTick = await onTick((payload) => state.set(payload));
    }
    if (!autosaveTimer) {
      autosaveTimer = setInterval(() => {
        const latest = get(state);
        if (!latest.paused) {
          saveSlot(0, 'Autosave').catch((err) => errorMessage.set(String(err)));
        }
      }, 60_000);
    }
    await emitTutorial();
  } catch (error) {
    errorMessage.set(String(error));
  } finally {
    loading.set(false);
  }
}

export async function setSpeed(speed: GameSpeed) {
  try {
    const snapshot = await changeSpeed(speed);
    state.set(snapshot);
    await recordMetric('speed_change', Date.now());
  } catch (error) {
    errorMessage.set(String(error));
  }
}

export async function setPaused(paused: boolean) {
  try {
    const snapshot = await togglePause(paused);
    state.set(snapshot);
  } catch (error) {
    errorMessage.set(String(error));
  }
}

export async function manualSave(slotId: number, name: string) {
  try {
    await saveSlot(slotId, name);
    await refreshSaves();
  } catch (error) {
    errorMessage.set(String(error));
  }
}

export async function manualLoad(slotId: number) {
  try {
    const snapshot = await loadSlot(slotId);
    state.set(snapshot);
  } catch (error) {
    errorMessage.set(String(error));
  }
}

export async function refreshSaves() {
  try {
    const slots = await listSaves();
    saves.set(slots);
  } catch (error) {
    errorMessage.set(String(error));
  }
}

export const gameState = state;
export const isLoading = loading;
export const error = errorMessage;
export const saveSlots = saves;

export function cleanup() {
  if (unsubscribeTick) {
    unsubscribeTick();
    unsubscribeTick = null;
  }
  if (autosaveTimer) {
    clearInterval(autosaveTimer);
    autosaveTimer = null;
  }
}

export async function buyGpu(rackId: string, template: Gpu) {
  try {
    const snapshot = await purchaseGpu(rackId, template);
    state.set(snapshot);
  } catch (error) {
    errorMessage.set(String(error));
  }
}
