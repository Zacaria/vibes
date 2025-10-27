use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyPreset {
    pub name: String,
    pub finance: FinancePreset,
    pub energy: EnergyPreset,
    pub market: MarketPreset,
    pub events: EventsPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancePreset {
    pub starting_cash: f32,
    pub success_cash_target: f32,
    pub sla_tolerance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyPreset {
    pub base_price_per_kwh: f32,
    pub carbon_target: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPreset {
    pub base_token_price: f32,
    pub demand_growth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsPreset {
    pub failure_multiplier: f32,
    pub random_seed: u64,
}

impl DifficultyPreset {
    pub fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn load_default() -> anyhow::Result<Self> {
        Self::load_from_file(Path::new("../configs/difficulty.json"))
    }
}
