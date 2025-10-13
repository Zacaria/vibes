# Self-Critique

1. **Simulation fidelity risk** – The scheduler currently distributes utilisation evenly across GPUs, which may under-represent per-rack bottlenecks. *Mitigation*: added unit tests ensuring inference workloads receive priority tokens, keeping regressions visible.
2. **Persistence coupling** – `persistence.rs` depends on `tauri-plugin-sql` internals. *Mitigation*: encapsulated all SQL usage behind helper functions and added a dedicated round-trip test using an in-memory pool to verify schema compatibility.
3. **Event frequency tuning** – Random incidents rely on hard-coded probabilities. *Mitigation*: probabilities are scaled by tick duration and difficulty presets can override values via `configs/difficulty.json`.
4. **UI responsiveness** – Autosave could block if a save takes long. *Mitigation*: autosave executes via command invocation and errors surface through the global error store, avoiding silent stalls.
5. **Carbon accounting accuracy** – Carbon intensity currently averages data center metrics. *Mitigation*: economy unit tests assert cost/uptime bounds, and telemetry recording is ready for future analytics to refine weighting.
6. **Mobile smoke coverage** – Scripts cannot launch emulators in CI environments. *Mitigation*: provided platform-specific smoke scripts that gate on environment variables (`ANDROID_HOME` or macOS check) to ensure developers verify builds locally.
7. **Difficulty data loading** – Runtime file reads could fail on mobile. *Mitigation*: `load_difficulty` falls back to a compiled default when the JSON read fails, guaranteeing boot.
8. **UI accessibility** – Limited settings toggles may miss some requirements. *Mitigation*: Settings screen includes language and colorblind toggles with high-contrast palette, and components avoid small text.
9. **Random event determinism** – RNG reseeding on load is not yet exposed. *Mitigation*: saves capture full state including incident history, and integration tests simulate multiple ticks to validate stable execution.
10. **Test runtime** – Integration tests rely on Tokio runtime and SQLx, increasing CI time. *Mitigation*: tests scope to minimal ticks and in-memory SQLite to keep execution under a second.
