use llm_model_farm_manager::game::balance::{
    DifficultyPreset, EnergyPreset, EventsPreset, FinancePreset, MarketPreset,
};
use llm_model_farm_manager::game::{scheduler, GameState};

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
fn inference_priority() {
    let preset = preset();
    let mut state = GameState::bootstrap(&preset);
    let report = scheduler::update_scheduler(&mut state, 5);
    assert!(report.inference_tokens_served >= report.training_tokens_processed);
}
