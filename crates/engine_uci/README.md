# engine_uci

`engine_uci` preserves the future Stockfish/UCI boundary without dragging subprocess management into the shipped local shell.

## Architecture

- `EngineRequest`, `EngineResponse`, and `EngineError` define the narrow controller contract that the rest of the workspace can depend on without knowing about a concrete engine process.
- `EngineController` stays intentionally small: health reporting plus one evaluation entrypoint.
- `MockEngineController` is the primary implementation in M3 so request validation and response mapping can be tested deterministically.

## Invariants

- Blank position strings and zero movetime requests are rejected before any controller-specific work happens.
- Tests cover controller behavior through the public trait boundary instead of relying on real UCI subprocesses.
- The mock must expose both unhealthy and scripted-failure paths so callers can exercise real error handling branches.

## Tradeoffs

- M3 favors deterministic controller tests over a real engine-process harness because honest coverage is cheaper and less brittle when the subprocess boundary stays out of scope.
- The trait surface is intentionally narrow so future UCI transport work can slot in without leaking process details across the workspace.
