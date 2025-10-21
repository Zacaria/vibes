use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::warn;

use super::{EventLogEntry, GameState};
use crate::game::economy::EconomyReport;
use crate::game::scheduler::SchedulerReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcome {
    pub title: String,
    pub description: String,
}

pub fn apply_random_events(
    state: &mut GameState,
    app_handle: &AppHandle,
    minutes_per_tick: u64,
    schedule: &SchedulerReport,
    economy: &EconomyReport,
) {
    let mut rng = rand::thread_rng();
    let minutes = minutes_per_tick as f32 / 60.0;

    for dc in &mut state.data_centers {
        for rack in &mut dc.racks {
            for gpu in &mut rack.gpus {
                let failure_probability = gpu.failure_rate * minutes;
                if rng.gen::<f32>() < failure_probability {
                    gpu.powered = false;
                    gpu.utilization = 0.0;
                    let event = EventOutcome {
                        title: "GPU Failure".to_string(),
                        description: format!(
                            "{} in {} failed. Replacement cost €{:.0}",
                            gpu.name,
                            rack.name,
                            gpu.cost_capex * 0.3
                        ),
                    };
                    state.cash -= (gpu.cost_capex * 0.3) as f64;
                    push_event(state, event);
                }
            }
        }
    }

    if rng.gen::<f32>() < 0.01 * minutes {
        let penalty = (1.0 - economy.uptime as f64) * 500.0;
        if penalty > 1.0 {
            state.cash -= penalty;
            push_event(
                state,
                EventOutcome {
                    title: "SLA Penalty".to_string(),
                    description: format!("Paid €{:.0} for SLA breach", penalty),
                },
            );
        }
    }

    if rng.gen::<f32>() < 0.02 * minutes {
        let delta_demand = rng.gen_range(0.1..0.25);
        for seg in &mut state.user_segments {
            seg.demand_tokens_per_hour *= 1.0 + delta_demand;
        }
        push_event(
            state,
            EventOutcome {
                title: "Demand Spike".to_string(),
                description: format!("User demand increased by {:.0}%", delta_demand * 100.0),
            },
        );
    }

    if schedule.sla_violations > 0 {
        warn!("sla violations={}", schedule.sla_violations);
    }

    let _ = app_handle.emit_all("game://tick", &state);
}

fn push_event(state: &mut GameState, event: EventOutcome) {
    let entry = EventLogEntry {
        ts_minutes: state.tick,
        title: event.title.clone(),
        description: event.description.clone(),
    };
    state.events.push(entry.clone());
    state.active_incidents.push(entry);
}
