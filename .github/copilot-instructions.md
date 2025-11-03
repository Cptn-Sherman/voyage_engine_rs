## Purpose

Short, focused instructions to help an AI code assistant be productive in this repository.

## Big-picture architecture (quick)
- This is a Bevy-based game engine written in Rust (`edition = 2021`). The executable is the `voyage_engine` crate (see `Cargo.toml`).
- Major runtime modules live under `src/`: `camera`, `player`, `terrain`, `user_interface`, `utils`, and top-level glue in `src/main.rs`.
- Systems are scheduled using Bevy stages: `PreStartup`, `Startup`, and `Update` in `main.rs`. Plugins (e.g. `PlayerPlugin`, `DebugInterfacePlugin`) encapsulate subsystem registration and their own resources/systems.

## How to build & run (developer workflows)
- Build: `cargo build` (workspace resolver = "2" is required — already set in `Cargo.toml`).
- Run (dev): `cargo run` from repository root. For CI or performance testing use `--release`.
- VS Code: there is a workspace task labelled `rust: cargo build` you can run from the command palette / Tasks panel.
- Use `RUST_LOG=info` or set env logging to see info/warn messages emitted by systems (many systems use `info!`/`warn!`).

## Project-specific conventions & patterns
- Configuration and input bindings are modeled as Bevy `Resource`s (see `src/config.rs`): `Bindings` and `EngineSettings` are Resources with `Default` implementations.
- Components and Resources: prefer `#[derive(Component)]` for entities and `#[derive(Resource)]` for global settings.
- Systems often accept `Res<Bindings>` or `ResMut<Assets<_>>` rather than passing raw primitives — follow that pattern when adding or modifying systems.
- Events/Messages: the project uses a message/event pattern (e.g. `ToggleCameraEvent` in `src/camera.rs`) — write systems that produce and consume these messages rather than using global state where reasonable.
- Camera switching example: `swap_camera_target` in `src/camera.rs` shows parenting/unparenting entities and writing a `ToggleCameraEvent` — reference this when creating other mode-switching features.

## Key files to inspect (examples)
- `src/main.rs` — plugin composition, system scheduling, and app startup flow (see `.add_plugins(...)` and `.add_systems(...)`).
- `src/camera.rs` — camera creation, free-camera motion, screenshot logic (`take_screenshot`) and audio for camera toggles.
- `src/config.rs` — input bindings and engine settings (defaults and how systems read them).
- `player/` — player controller and action modules (movement, crouch, sprint patterns).
- `terrain/` — voxel/terrain meshing code and chunk management (transvoxel, chunk meshing patterns).
- `assets/` — audio, models and textures used by the engine (paths are referenced directly by asset server loads like `audio/Blip-003.wav`).

## Useful code patterns to follow
- System grouping: register multiple related systems together using tuple chaining in `add_systems(Startup, (...).chain())` as in `main.rs`.
- Resource initialization: use `.init_resource::<T>()` and then `insert_resource(...)` in `main.rs` for custom defaults.
- Mesh/material creation: use `ResMut<Assets<Mesh>>` and `ResMut<Assets<StandardMaterial>>` passed into setup systems, and wrap materials in project helper types like `MeshMaterial3d`.
- Audio: Bevy Kira audio is used; audio handles are stored as `Resource`s (see camera sound fx loaders) and played via the `Audio` resource.

## Integration & dependencies
- Bevy (0.17) is the core engine with features: `dynamic_linking`, `file_watcher`, `embedded_watcher` (see `Cargo.toml`). Keep these features when changing the Bevy version.
- Other notable crates: `avian3d` for 3D utilities, `transvoxel` for voxel terrain, `bevy_kira_audio`, and `bevy_turborand`.

## Quick examples (copyable patterns)
- Add a resource with a default:

  - See `main.rs`: `.init_resource::<Bindings>()` and `.insert_resource(EngineSettings { ..default() })`.

- Register a simple startup system:

  - In `main.rs`: `.add_systems(Startup, (setup, start_background_audio, ...).chain())`.

## Notes and gotchas
- `Cargo.toml` sets a workspace `resolver = "2"` — do not remove this; it affects wgpu/Bevy resolution.
- `RenderAssetBytesPerFrame` is set large in `main.rs`; the app config intentionally increases asset throughput for heavy scenes.
- Screenshots: `take_screenshot` writes to disk with `EngineSettings.screenshot_format` — changing the format requires updating `get_valid_extension` behavior in `src/utils`.

## When in doubt
- Search for `add_plugins`, `add_systems`, and `init_resource` to find where a feature should be integrated.
- Prefer to mimic existing systems' signature patterns (`Query`, `Res`, `ResMut`, `MessageReader/Writer`) when adding new code.

---
If anything above is unclear or you'd like more detail (examples for a specific subsystem, or a recommended test harness), tell me which area to expand and I'll update this file.
