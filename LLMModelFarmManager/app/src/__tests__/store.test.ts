import { describe, expect, it, vi, beforeEach } from 'vitest';
import { gameState, initGame, setSpeed } from '../lib/store';
import type { GameState } from '../lib/types';

vi.mock('../lib/api', () => {
  const state: GameState = {
    tick: 1,
    speed: 'Normal',
    paused: false,
    cash: 1_000_000,
    profit_per_hour: 100_000,
    tokens_served_per_hour: 200_000,
    uptime: 0.99,
    carbon_intensity: 280,
    power_draw_kw: 400,
    data_centers: [],
    models: [],
    workloads: [],
    token_providers: [],
    user_segments: [],
    ledger: [],
    events: [],
    active_incidents: []
  };
  return {
    fetchState: vi.fn().mockResolvedValue(state),
    listSaves: vi.fn().mockResolvedValue([]),
    onTick: vi.fn().mockResolvedValue(() => undefined),
    emitTutorial: vi.fn().mockResolvedValue(undefined),
    changeSpeed: vi.fn().mockImplementation(async (speed) => ({ ...state, speed })),
    recordMetric: vi.fn().mockResolvedValue(undefined),
    togglePause: vi.fn().mockResolvedValue(state),
    saveSlot: vi.fn().mockResolvedValue(undefined)
  };
});

beforeEach(() => {
  vi.useFakeTimers();
  const defaultState = {
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
  } satisfies GameState;
  gameState.set(defaultState);
  vi.clearAllMocks();
  vi.clearAllTimers();
});

describe('store', () => {
  it('initializes state from backend', async () => {
    await initGame();
    const value = getStore();
    expect(value.cash).toBe(1_000_000);
  });

  it('updates speed via API', async () => {
    await initGame();
    await setSpeed('Double');
    const value = getStore();
    expect(value.speed).toBe('Double');
  });
});

function getStore() {
  let current: GameState | undefined;
  const unsubscribe = gameState.subscribe((value) => (current = value));
  unsubscribe();
  return current!;
}
