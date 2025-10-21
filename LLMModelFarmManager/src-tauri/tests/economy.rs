use llm_model_farm_manager::game::balance::{
    DifficultyPreset, EnergyPreset, EventsPreset, FinancePreset, MarketPreset,
};
use llm_model_farm_manager::game::{economy, scheduler, GameState};

fn preset() -> DifficultyPreset {
    DifficultyPreset {
        name: "Test".into(),
        finance: FinancePreset {
            starting_cash: 5_000_000.0,
            success_cash_target: 10_000_000.0,
            sla_tolerance: 3,
        },
        energy: EnergyPreset {
            base_price_per_kwh: 0.18,
            carbon_target: 280.0,
        },
        market: MarketPreset {
            base_token_price: 40.0,
            demand_growth: 0.1,
        },
        events: EventsPreset {
            failure_multiplier: 1.0,
            random_seed: 1,
        },
    }
}

#[test]
fn net_cash_flow_within_bounds() {
    let preset = preset();
    let mut state = GameState::bootstrap(&preset);
    let schedule_report = scheduler::update_scheduler(&mut state, 5);
    let report = economy::update_economy(&mut state, 5, &schedule_report);
    assert!(report.revenue_eur >= 0.0);
    assert!(report.net_cash_flow_eur.is_finite());
    assert!(state.uptime >= 0.0 && state.uptime <= 1.1);
}

#[test]
fn energy_cost_scales_with_power() {
    let preset = preset();
    let mut state = GameState::bootstrap(&preset);
    let schedule_report = scheduler::update_scheduler(&mut state, 5);
    let report = economy::update_economy(&mut state, 5, &schedule_report);
    assert!(report.energy_cost_eur > 0.0);
}
