export interface GpuHardware {
  arch: string;
  vram_gb: number;
  tdp_w: number;
}

export interface Gpu {
  id: string;
  name: string;
  hw: GpuHardware;
  efficiency_tok_per_joule: number;
  cost_capex: number;
  failure_rate: number;
  hours_until_failure: number;
  powered: boolean;
  utilization: number;
}

export interface Rack {
  id: string;
  name: string;
  slots: number;
  gpus: Gpu[];
}

export interface PowerContract {
  id: string;
  name: string;
  price_eur_per_kwh: number;
  peak_surcharge: number;
  renewable_share: number;
  curtailment_events: number;
}

export interface Cooling {
  id: string;
  name: string;
  cap_kw: number;
  opex_per_kw: number;
  efficiency: number;
}

export interface DataCenter {
  id: string;
  name: string;
  region: string;
  pue: number;
  grid_carbon_intensity: number;
  racks: Rack[];
  cooling: Cooling;
  power_contract: PowerContract;
}

export interface LlmModel {
  id: string;
  family: string;
  size_b: number;
  train_cost_per_token: number;
  infer_cost_per_token: number;
  quality_score: number;
}

export type WorkloadKind = 'Training' | 'Inference';

export interface Workload {
  id: string;
  kind: WorkloadKind;
  model_id: string;
  tokens_per_hour_target: number;
  qos_sla_ms: number;
  active: boolean;
}

export interface TokenProvider {
  id: string;
  name: string;
  price_per_1k_tokens: number;
  latency_ms: number;
  reliability: number;
}

export interface UserSegment {
  id: string;
  name: string;
  size: number;
  demand_tokens_per_hour: number;
  churn_rate: number;
  price_per_1k_tokens: number;
}

export interface LedgerEntry {
  ts_minutes: number;
  description: string;
  delta_cash: number;
  balance: number;
}

export interface EventLogEntry {
  ts_minutes: number;
  title: string;
  description: string;
}

export type GameSpeed = 'Normal' | 'Double' | 'Quadruple';

export interface GameState {
  tick: number;
  speed: GameSpeed;
  paused: boolean;
  cash: number;
  profit_per_hour: number;
  tokens_served_per_hour: number;
  uptime: number;
  carbon_intensity: number;
  power_draw_kw: number;
  data_centers: DataCenter[];
  models: LlmModel[];
  workloads: Workload[];
  token_providers: TokenProvider[];
  user_segments: UserSegment[];
  ledger: LedgerEntry[];
  events: EventLogEntry[];
  active_incidents: EventLogEntry[];
}

export interface SaveSlotSummary {
  id: number;
  name: string;
  created_at: string;
}
