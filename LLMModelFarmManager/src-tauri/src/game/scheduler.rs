use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{GameState, WorkloadKind};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerReport {
    pub inference_tokens_served: f32,
    pub inference_tokens_requested: f32,
    pub training_tokens_processed: f32,
    pub power_draw_kw: f32,
    pub sla_violations: u32,
}

pub fn update_scheduler(state: &mut GameState, minutes_per_tick: u64) -> SchedulerReport {
    let mut report = SchedulerReport::default();
    if state.data_centers.is_empty() {
        return report;
    }

    let minutes = minutes_per_tick as f32;
    let total_capacity_tokens: f32 = state
        .data_centers
        .iter()
        .flat_map(|dc| &dc.racks)
        .flat_map(|rack| &rack.gpus)
        .map(|gpu| {
            let joules = gpu.hw.tdp_w * 60.0 * minutes;
            joules * gpu.efficiency_tok_per_joule
        })
        .sum();

    let mut remaining_capacity = total_capacity_tokens;
    let mut served_inference = 0.0;
    let mut requested_inference = 0.0;
    let mut served_training = 0.0;

    for workload in state.workloads.iter().filter(|w| w.active) {
        let demand_tokens = workload.tokens_per_hour_target * (minutes / 60.0);
        match workload.kind {
            WorkloadKind::Inference => {
                requested_inference += demand_tokens;
                let served = demand_tokens.min(remaining_capacity);
                served_inference += served;
                remaining_capacity -= served;
                if served + f32::EPSILON < demand_tokens {
                    report.sla_violations += 1;
                }
            }
            WorkloadKind::Training => {
                let served = demand_tokens.min(remaining_capacity);
                served_training += served;
                remaining_capacity -= served;
            }
        }
        if remaining_capacity <= 0.0 {
            break;
        }
    }

    let utilization = if total_capacity_tokens.abs() < f32::EPSILON {
        0.0
    } else {
        1.0 - (remaining_capacity / total_capacity_tokens)
    };

    let mut power_draw_kw = 0.0;
    for dc in &mut state.data_centers {
        for rack in &mut dc.racks {
            for gpu in &mut rack.gpus {
                gpu.utilization = utilization;
                gpu.powered = utilization > 0.01;
                power_draw_kw += (gpu.hw.tdp_w * gpu.utilization) / 1000.0;
            }
        }
    }

    debug!(
        "scheduler utilization={} power_kw={}",
        utilization, power_draw_kw
    );

    report.power_draw_kw = power_draw_kw;
    report.inference_tokens_served = served_inference;
    report.inference_tokens_requested = requested_inference;
    report.training_tokens_processed = served_training;

    report
}
