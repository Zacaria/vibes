# Test Plan

## Automated Coverage
- **Rust unit tests** (`cargo test`): economy math, scheduler priority, persistence round-trip, smoke playthrough.
- **Clippy** (`cargo clippy -- -D warnings`): zero-warning policy.
- **Vitest** (`pnpm -C app test`): store reactivity and command wiring.
- **CI**: see `.github/workflows/ci.yml` for combined build + test pipeline.

## Manual Desktop QA
1. `pnpm -C app install && pnpm -C app dev` in one terminal, `cargo tauri dev` in another.
2. Verify dashboard KPIs update every tick and autosave writes slot 0.
3. Pause/resume via sidebar button and space bar shortcut.
4. Purchase a GPU in Market view; rack utilisation increases and ledger reflects purchase.
5. Trigger tutorial via settings (on app load) and ensure toast/events appear.
6. Verify saves list updates after manual save.

## Android Emulator Smoke
1. Set `ANDROID_HOME`, install SDK 34 and NDK r26.
2. Run `./scripts/smoke_android.sh` (requires an emulator named `pixel_6`).
3. Confirm CLI exits successfully and game state logs appear in `cargo` output.

## iOS Simulator Smoke
1. On macOS with Xcode 15+, run `./scripts/smoke_ios.sh`.
2. Ensure the simulator boots the app (background launch) and CLI terminates after 20s.

## Performance & Battery Checks
- Run the app for 10 real-time minutes at 4× speed; ensure CPU usage stays <60% and no dropped frames appear (profiling via platform tools).
- Toggle pause on low-battery devices to confirm throttling logic in UI reacts quickly.

## Accessibility & Localization
- Switch Settings → Language to Français and ensure UI strings switch (future improvement; placeholder indicates planned coverage).
- Enable colorblind mode and verify charts remain distinguishable.

## Failure Triage
- Capture logs via `tauri-plugin-log` output (desktop) or platform logcat/syslog.
- Persist problematic states using manual saves and attach JSON payload for reproduction.
