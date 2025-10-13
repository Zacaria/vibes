use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

pub mod balance;
pub mod economy;
pub mod events;
pub mod persistence;
pub mod scheduler;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameSpeed {
    Normal,
    Double,
    Quadruple,
}

impl GameSpeed {
    pub fn multiplier(&self) -> u32 {
        match self {
            GameSpeed::Normal => 1,
            GameSpeed::Double => 2,
            GameSpeed::Quadruple => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCenter {
    pub id: Uuid,
    pub name: String,
    pub region: String,
    pub pue: f32,
    pub grid_carbon_intensity: f32,
    pub racks: Vec<Rack>,
    pub cooling: Cooling,
    pub power_contract: PowerContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rack {
    pub id: Uuid,
    pub name: String,
    pub slots: u32,
    pub gpus: Vec<Gpu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuHardware {
    pub arch: String,
    pub vram_gb: u32,
    pub tdp_w: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gpu {
    pub id: Uuid,
    pub name: String,
    pub hw: GpuHardware,
    pub efficiency_tok_per_joule: f32,
    pub cost_capex: f32,
    pub failure_rate: f32,
    pub hours_until_failure: f32,
    pub powered: bool,
    pub utilization: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: Uuid,
    pub family: String,
    pub size_b: u64,
    pub train_cost_per_token: f32,
    pub infer_cost_per_token: f32,
    pub quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub id: Uuid,
    pub kind: WorkloadKind,
    pub model_id: Uuid,
    pub tokens_per_hour_target: f32,
    pub qos_sla_ms: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadKind {
    Training,
    Inference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenProvider {
    pub id: Uuid,
    pub name: String,
    pub price_per_1k_tokens: f32,
    pub latency_ms: f32,
    pub reliability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerContract {
    pub id: Uuid,
    pub name: String,
    pub price_eur_per_kwh: f32,
    pub peak_surcharge: f32,
    pub renewable_share: f32,
    pub curtailment_events: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooling {
    pub id: Uuid,
    pub name: String,
    pub cap_kw: f32,
    pub opex_per_kw: f32,
    pub efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSegment {
    pub id: Uuid,
    pub name: String,
    pub size: f32,
    pub demand_tokens_per_hour: f32,
    pub churn_rate: f32,
    pub price_per_1k_tokens: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub ts_minutes: u64,
    pub description: String,
    pub delta_cash: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub ts_minutes: u64,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub tick_seconds: u64,
    pub minutes_per_tick: u64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tick_seconds: 1,
            minutes_per_tick: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tick: u64,
    pub speed: GameSpeed,
    pub paused: bool,
    pub cash: f64,
    pub profit_per_hour: f64,
    pub tokens_served_per_hour: f64,
    pub uptime: f32,
    pub carbon_intensity: f32,
    pub power_draw_kw: f32,
    pub data_centers: Vec<DataCenter>,
    pub models: Vec<LlmModel>,
    pub workloads: Vec<Workload>,
    pub token_providers: Vec<TokenProvider>,
    pub user_segments: Vec<UserSegment>,
    pub ledger: Vec<LedgerEntry>,
    pub events: Vec<EventLogEntry>,
    pub active_incidents: Vec<EventLogEntry>,
}

impl GameState {
    pub fn bootstrap(difficulty: &balance::DifficultyPreset) -> Self {
        let token_provider = TokenProvider {
            id: Uuid::new_v4(),
            name: "DefaultTokens".to_string(),
            price_per_1k_tokens: difficulty.market.base_token_price,
            latency_ms: 45.0,
            reliability: 0.995,
        };

        let power_contract = PowerContract {
            id: Uuid::new_v4(),
            name: "GreenGrid".to_string(),
            price_eur_per_kwh: difficulty.energy.base_price_per_kwh,
            peak_surcharge: 0.08,
            renewable_share: 0.65,
            curtailment_events: 0.01,
        };

        let cooling = Cooling {
            id: Uuid::new_v4(),
            name: "LiquidCool".to_string(),
            cap_kw: 500.0,
            opex_per_kw: 0.04,
            efficiency: 0.92,
        };

        let gpu = Gpu {
            id: Uuid::new_v4(),
            name: "A100".to_string(),
            hw: GpuHardware {
                arch: "Ampere".to_string(),
                vram_gb: 80,
                tdp_w: 400.0,
            },
            efficiency_tok_per_joule: 0.62,
            cost_capex: 12000.0,
            failure_rate: 0.0001,
            hours_until_failure: 5000.0,
            powered: true,
            utilization: 0.0,
        };

        let rack = Rack {
            id: Uuid::new_v4(),
            name: "Rack-1".to_string(),
            slots: 8,
            gpus: vec![gpu],
        };

        let datacenter = DataCenter {
            id: Uuid::new_v4(),
            name: "FoundersDC".to_string(),
            region: "eu-west".to_string(),
            pue: 1.2,
            grid_carbon_intensity: 280.0,
            racks: vec![rack],
            cooling,
            power_contract,
        };

        let model = LlmModel {
            id: Uuid::new_v4(),
            family: "Aurora".to_string(),
            size_b: 13_000_000_000,
            train_cost_per_token: 0.0004,
            infer_cost_per_token: 0.00008,
            quality_score: 0.78,
        };

        let workload_infer = Workload {
            id: Uuid::new_v4(),
            kind: WorkloadKind::Inference,
            model_id: model.id,
            tokens_per_hour_target: 500_000.0,
            qos_sla_ms: 200,
            active: true,
        };

        let workload_train = Workload {
            id: Uuid::new_v4(),
            kind: WorkloadKind::Training,
            model_id: model.id,
            tokens_per_hour_target: 200_000.0,
            qos_sla_ms: 500,
            active: true,
        };

        let enterprise_segment = UserSegment {
            id: Uuid::new_v4(),
            name: "Enterprise".to_string(),
            size: 35.0,
            demand_tokens_per_hour: 320_000.0,
            churn_rate: 0.02,
            price_per_1k_tokens: 45.0,
        };

        let ledger = vec![LedgerEntry {
            ts_minutes: 0,
            description: "Initial funding".to_string(),
            delta_cash: difficulty.finance.starting_cash as f64,
            balance: difficulty.finance.starting_cash as f64,
        }];

        GameState {
            tick: 0,
            speed: GameSpeed::Normal,
            paused: false,
            cash: difficulty.finance.starting_cash as f64,
            profit_per_hour: 0.0,
            tokens_served_per_hour: 0.0,
            uptime: 1.0,
            carbon_intensity: datacenter.grid_carbon_intensity,
            power_draw_kw: 0.0,
            data_centers: vec![datacenter],
            models: vec![model],
            workloads: vec![workload_infer, workload_train],
            token_providers: vec![token_provider],
            user_segments: vec![enterprise_segment],
            ledger,
            events: Vec::new(),
            active_incidents: Vec::new(),
        }
    }

    pub fn set_speed(&mut self, speed: GameSpeed) {
        self.speed = speed;
    }

    pub fn toggle_pause(&mut self, paused: bool) {
        self.paused = paused;
    }
}

#[derive(Clone)]
pub struct GameEngine {
    state: Arc<RwLock<GameState>>,
    config: GameConfig,
    running: Arc<AtomicBool>,
    tx: broadcast::Sender<GameState>,
    handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl GameEngine {
    pub fn new(difficulty: balance::DifficultyPreset) -> Self {
        let state = GameState::bootstrap(&difficulty);
        let (tx, _rx) = broadcast::channel(32);
        Self {
            state: Arc::new(RwLock::new(state)),
            config: GameConfig::default(),
            running: Arc::new(AtomicBool::new(false)),
            tx,
            handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn state(&self) -> Arc<RwLock<GameState>> {
        Arc::clone(&self.state)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GameState> {
        self.tx.subscribe()
    }

    pub fn snapshot(&self) -> GameState {
        self.state.read().clone()
    }

    pub fn start(&self, app_handle: tauri::AppHandle) {
        let app_handle = app_handle.clone();
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let state = Arc::clone(&self.state);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.tick_seconds));
            loop {
                interval.tick().await;
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                {
                    let mut guard = state.write();
                    if guard.paused {
                        continue;
                    }
                    let multiplier = guard.speed.multiplier() as u64;
                    for _ in 0..multiplier {
                        guard.tick += 1;
                        let schedule_report =
                            scheduler::update_scheduler(&mut *guard, config.minutes_per_tick);
                        let economy_report = economy::update_economy(
                            &mut *guard,
                            config.minutes_per_tick,
                            &schedule_report,
                        );
                        events::apply_random_events(
                            &mut *guard,
                            &app_handle,
                            config.minutes_per_tick,
                            &schedule_report,
                            &economy_report,
                        );
                    }
                    let _ = tx.send(guard.clone());
                }
            }
        });

        *self.handle.write() = Some(handle);
    }

    pub fn stop(&self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }
        if let Some(handle) = self.handle.write().take() {
            tokio::spawn(async move {
                handle.abort();
            });
        }
    }

    pub fn set_speed(&self, speed: GameSpeed) {
        let mut guard = self.state.write();
        guard.set_speed(speed);
    }

    pub fn set_paused(&self, paused: bool) {
        let mut guard = self.state.write();
        guard.toggle_pause(paused);
    }

    pub fn purchase_gpu(&self, rack_id: Uuid, template: &Gpu) -> AppResult<GameState> {
        let mut guard = self.state.write();
        let mut found = false;
        for dc in &mut guard.data_centers {
            for rack in &mut dc.racks {
                if rack.id == rack_id {
                    if rack.gpus.len() as u32 >= rack.slots {
                        return Err(AppError::InvalidRequest("Rack is full".into()));
                    }
                    guard.cash -= template.cost_capex as f64;
                    guard.ledger.push(LedgerEntry {
                        ts_minutes: guard.tick * self.config.minutes_per_tick,
                        description: format!("Purchased GPU {}", template.name),
                        delta_cash: -(template.cost_capex as f64),
                        balance: guard.cash,
                    });
                    let mut gpu = template.clone();
                    gpu.id = Uuid::new_v4();
                    rack.gpus.push(gpu);
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            return Err(AppError::InvalidRequest("Rack not found".into()));
        }
        Ok(guard.clone())
    }
}

impl Drop for GameEngine {
    fn drop(&mut self) {
        info!("dropping game engine");
        self.running.store(false, Ordering::SeqCst);
    }
}
