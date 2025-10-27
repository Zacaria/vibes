# LLM Model Farm Manager

LLM Model Farm Manager is a cross-platform management simulation game built with Rust and Tauri v2. Manage data centers, GPU racks, and workloads to keep your AI compute farm profitable while meeting sustainability goals.

## Prerequisites
- Rust 1.80+
- pnpm 8+
- Node.js 18+
- Tauri prerequisites (see https://tauri.app for platform-specific setup)

## Desktop Development
```bash
cd src-tauri
cargo install cargo-make --locked
cargo make setup
cargo tauri dev
```

## Android Build
Ensure Android SDK 34, NDK r26, Java 17, and Gradle are installed.
```bash
./scripts/build_android.sh
```

## iOS Build
Requires macOS with Xcode 15+ and Cocoapods.
```bash
./scripts/build_ios.sh
```

See `docs/TEST_PLAN.md` for the full testing matrix and `docs/DESIGN.md` for architecture details.
