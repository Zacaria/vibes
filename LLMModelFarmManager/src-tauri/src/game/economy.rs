use serde::{Deserialize, Serialize};
use tracing::info;

use super::{GameState, LedgerEntry};
use crate::game::scheduler::SchedulerReport;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EconomyReport {
    pub revenue_eur: f64,
    pub energy_cost_eur: f64,
    pub token_cost_eur: f64,
    pub depreciation_eur: f64,
    pub maintenance_eur: f64,
    pub net_cash_flow_eur: f64,
    pub uptime: f32,
    pub carbon_intensity: f32,
}

pub fn update_economy(
    state: &mut GameState,
    minutes_per_tick: u64,
    schedule: &SchedulerReport,
) -> EconomyReport {
    let mut report = EconomyReport::default();
    if state.data_centers.is_empty() {
        return report;
    }

    let hours = minutes_per_tick as f32 / 60.0;
    let served_tokens = schedule.inference_tokens_served;
    let requested_tokens = schedule.inference_tokens_requested.max(1.0);
    let uptime_ratio = (served_tokens / requested_tokens).clamp(0.0, 1.0);

    let avg_price = state
        .user_segments
        .iter()
        .map(|seg| seg.price_per_1k_tokens * seg.size)
        .sum::<f32>()
        / state.user_segments.len().max(1) as f32;

    let revenue = (served_tokens as f64 / 1000.0) * avg_price as f64;

    let total_power_kw = schedule.power_draw_kw
        * state.data_centers.iter().map(|dc| dc.pue).sum::<f32>()
        / state.data_centers.len().max(1) as f32;

    let avg_price_per_kwh = state
        .data_centers
        .iter()
        .map(|dc| dc.power_contract.price_eur_per_kwh)
        .sum::<f32>()
        / state.data_centers.len().max(1) as f32;

    let energy_cost = total_power_kw as f64 * avg_price_per_kwh as f64 * hours as f64;

    let token_price = state
        .token_providers
        .first()
        .map(|p| p.price_per_1k_tokens as f64)
        .unwrap_or(0.0);

    let token_cost = (served_tokens as f64 / 1000.0) * token_price;

    let depreciation = state
        .data_centers
        .iter()
        .flat_map(|dc| &dc.racks)
        .flat_map(|rack| &rack.gpus)
        .map(|gpu| gpu.cost_capex as f64 / (5.0 * 365.0 * 24.0 / hours as f64))
        .sum::<f64>();

    let maintenance = 0.01 * revenue;

    let net = revenue - energy_cost - token_cost - depreciation - maintenance;

    state.cash += net;
    state.profit_per_hour = net / hours as f64;
    state.tokens_served_per_hour = served_tokens as f64 / hours as f64;
    state.uptime = 0.9 * state.uptime + 0.1 * uptime_ratio;
    state.power_draw_kw = total_power_kw;
    state.carbon_intensity = state
        .data_centers
        .iter()
        .map(|dc| dc.grid_carbon_intensity)
        .sum::<f32>()
        / state.data_centers.len().max(1) as f32;

    report.revenue_eur = revenue;
    report.energy_cost_eur = energy_cost;
    report.token_cost_eur = token_cost;
    report.depreciation_eur = depreciation;
    report.maintenance_eur = maintenance;
    report.net_cash_flow_eur = net;
    report.uptime = state.uptime;
    report.carbon_intensity = state.carbon_intensity;

    state.ledger.push(LedgerEntry {
        ts_minutes: state.tick * minutes_per_tick,
        description: "Operations".to_string(),
        delta_cash: net,
        balance: state.cash,
    });

    info!("tick={} revenue={:.2} net={:.2}", state.tick, revenue, net);

    report
}
