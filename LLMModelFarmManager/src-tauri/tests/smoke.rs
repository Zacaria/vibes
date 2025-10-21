use llm_model_farm_manager::game::balance::{
    DifficultyPreset, EnergyPreset, EventsPreset, FinancePreset, MarketPreset,
};
use llm_model_farm_manager::game::{economy, scheduler, GameEngine, GameState};

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

#[tokio::test]
async fn simulate_ticks_and_purchase() {
    let preset = preset();
    let mut state = GameState::bootstrap(&preset);
    for _ in 0..5 {
        let schedule = scheduler::update_scheduler(&mut state, 5);
        let _economy = economy::update_economy(&mut state, 5, &schedule);
    }
    assert!(state.cash > 0.0);

    let rack_id = state.data_centers[0].racks[0].id;
    let template = state.data_centers[0].racks[0].gpus[0].clone();
    let engine = GameEngine::new(preset());
    engine.state().write().data_centers = state.data_centers.clone();
    engine
        .purchase_gpu(rack_id, &template)
        .expect("purchase works");
}
