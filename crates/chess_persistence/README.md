# chess_persistence

`chess_persistence` owns the file-backed save boundary for manual saves, interrupted-session recovery, and the small M3 shell settings contract.

## Architecture

- Snapshots persist `chess_core::GameState` plus only the shell metadata that affects legal resume behavior, such as pending promotion, selected square, last move, claimed draw state, and recovery dirtiness.
- `SessionStore` is the only place that knows the on-disk layout, platform app-data root, atomic JSON writes, and slot-id validation rules.
- Runtime storage defaults to the standard app-data directory for the platform, while tests and tooling inject roots explicitly for deterministic filesystem behavior.
- Manual saves use stable player-managed slot ids and labels, while interrupted-session recovery uses a dedicated autosave record so the app can restore the last interrupted match without touching user history.
- Shell settings stay intentionally narrow in M3: startup recovery policy, destructive-action confirmations, and display mode.

## Invariants

- The crate never serializes Bevy ECS state or UI entities.
- Recovery and manual save flows cannot overwrite each other by accident.
- Loaded manual saves derive slot identity from the file path boundary instead of trusting serialized slot ids inside the JSON payload.
- Filesystem tests use temporary directories rather than project-relative paths.

## Tradeoffs

- JSON and versioned snapshot structs keep the persisted format inspectable and migration-friendly.
- A single repository type owns saves, recovery state, and settings so path policy and atomic writes stay in one place.
- App-data defaults match desktop expectations, while injected roots keep tests and packaging smoke reproducible.
