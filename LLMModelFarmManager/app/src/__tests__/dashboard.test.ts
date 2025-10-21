import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Dashboard from '../routes/Dashboard.svelte';
import { readable } from 'svelte/store';
import type { GameState } from '../lib/types';

const mockState: GameState = {
  tick: 1,
  speed: 'Normal',
  paused: false,
  cash: 2_000_000,
  profit_per_hour: 150_000,
  tokens_served_per_hour: 300_000,
  uptime: 0.995,
  carbon_intensity: 280,
  power_draw_kw: 420,
  data_centers: [],
  models: [],
  workloads: [],
  token_providers: [],
  user_segments: [],
  ledger: [],
  events: [],
  active_incidents: []
};

describe('Dashboard', () => {
  it('renders KPI cards', () => {
    const { getByText } = render(Dashboard, {
      props: {
        state: mockState,
        kpiStore: readable({
          cash: mockState.cash,
          profit: mockState.profit_per_hour,
          tokens: mockState.tokens_served_per_hour,
          uptime: mockState.uptime,
          carbon: mockState.carbon_intensity,
          power: mockState.power_draw_kw,
          speed: mockState.speed,
          paused: mockState.paused
        })
      }
    });
    expect(getByText('Cash')).toBeTruthy();
    expect(getByText('€2,000,000')).toBeTruthy();
  });
});
