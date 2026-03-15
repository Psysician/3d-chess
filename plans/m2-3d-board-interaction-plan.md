# Plan

## Overview

Implement M2 so the Bevy shell becomes a complete local human-vs-human 3D chess match by wiring game_app to the M1 chess_core domain, replacing hardcoded board visuals with GameState-driven presentation, and adding input, promotion, and result flow.

**Approach**: Land M2 in three sequential milestones: establish a MatchSession and minimal screen-flow foundation, replace shell-only rendering with domain-driven board/piece sync plus internal board picking, then close the loop with HUD/promotion/result feedback and Bevy integration tests. Keep chess_core authoritative, continue using procedural meshes for this milestone, and defer save/load UX and AI integration to M3 and M4.

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Introduce a dedicated MatchSession resource as the only Bevy-to-domain bridge. | M1 already made chess_core authoritative -> M2 needs game_app orchestration without copying rules into ECS -> one MatchSession resource keeps GameState, legal move queries, and result status centralized. |
| DL-002 | Render board occupancy by synchronizing Bevy piece entities from GameState instead of letting entities become the source of truth. | The current scene hardcodes pieces independently of the domain -> local play needs visuals to match chess_core exactly after every move -> render sync must derive from GameState snapshots rather than local transform mutation logic. |
| DL-003 | Implement board picking with internal camera-ray to board-square mapping rather than adding a third-party picking dependency. | The board is a regular 8x8 plane with stable coordinates -> M2 needs deterministic square selection more than generalized scene picking -> internal math keeps the dependency surface small and tied to existing board_axis geometry. |
| DL-004 | Use a minimal MainMenu -> MatchLoading -> InMatch -> MatchResult state path for M2 and defer broader shell flows to M3. | M2 acceptance requires a playable local match inside the production shell -> full setup/settings/recovery flows belong to M3 -> a narrow state path delivers end-to-end play now without prematurely expanding menu scope. |
| DL-005 | Handle promotion through an explicit pending-promotion overlay with mouse and keyboard piece choice before move application completes. | Promotion is mandatory for a complete local match -> auto-queen would break rules completeness and later UX consistency -> a pending-promotion state keeps M2 correct while remaining lightweight. |
| DL-006 | Expose a claim-draw action in the in-match HUD when chess_core reports a claimable draw state. | M1 models threefold and fifty-move draws as claimable rather than automatic -> a full local match can stall without a way to claim them -> M2 should surface a simple claim action instead of changing domain semantics. |
| DL-007 | Keep procedural board and piece silhouettes for M2 and invest polish in materials, highlights, camera, and motion rather than authored art assets. | The current shell already has a strong procedural visual baseline -> authored models would widen scope into art-pipeline work -> M2 should ship interactive readability and motion polish while preserving the existing aesthetic direction. |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| Drive local gameplay directly from Bevy entity state. | The architecture lock says chess_core remains authoritative, so entity state may mirror but must not own move legality or match outcome. (ref: DL-001) |
| Keep the current hardcoded piece layout and only mutate visible transforms after moves. | That would preserve the current domain/view split bug and makes promotion, capture, restore, and result handling drift-prone. (ref: DL-002) |
| Add a general-purpose picking crate for square selection. | M2 only needs deterministic mapping from the existing camera and board plane to chess squares, so an extra dependency would add more surface than value. (ref: DL-003) |
| Defer promotion and draw-claim UX until M3. | A local match is not complete if promotion and claimable draws cannot be resolved inside the shipped M2 loop. (ref: DL-005) |
| Introduce authored 3D models during M2. | The milestone goal is interactive playability and readable polish, not an art-pipeline expansion that belongs later. (ref: DL-007) |

### Constraints

- MUST: implement local human-vs-human play only; AI flow remains reserved for M4
- MUST: keep chess_core as the authoritative rules source and treat Bevy as presentation plus orchestration only
- MUST: support mouse and keyboard input on Windows and Linux targets inside the native Rust + Bevy app
- MUST-NOT: expand M2 into save/load UX, settings, or broader product-shell hardening beyond the minimum playable local-match path
- SHOULD: preserve the current production-intent shell look by building on existing board materials, camera framing, and UI tone instead of replacing them with placeholders

### Known Risks

- **Square-to-world coordinate drift could make picking, highlighting, and piece placement disagree visually.**: Introduce one shared board-coordinate module and use it for board_scene, piece_view, and input mapping.
- **Selection logic may duplicate chess rules if it locally decides what targets are legal.**: Derive every preview and execution path from GameState::legal_moves and MatchSession-derived domain queries only.
- **Promotion can dead-end the match if the UI applies a move before a piece choice is provided.**: Represent promotion as a pending interaction state and block move commit until the user chooses a legal promotion piece.
- **Claimable draw states may be invisible to players, leaving some matches impossible to conclude under the current domain semantics.**: Expose an in-match claim-draw affordance whenever chess_core reports draw_available.is_claimable().
- **Trying to deliver authored art or full shell flows in M2 could delay the first playable local match.**: Keep procedural visuals, narrow the state path, and reserve save/load shell work for M3.

## Invisible Knowledge

### System

3d-chess is a Rust workspace where chess_core owns all chess legality and game status while game_app owns Bevy rendering, input, UI, and scene orchestration. M2 is the first milestone that must make those layers meet without violating that boundary.

### Invariants

- GameState in chess_core remains the single source of truth for piece placement, legal targets, check state, promotion requirements, and outcomes.
- Every visible piece, highlight, and HUD status in game_app must be derived from MatchSession and chess_core, never from independent Bevy-only rule state.
- Board square coordinates must be computed from one canonical mapping shared by rendering and picking code.
- M2 stops at a complete local playable loop; save/load UX, settings, and Stockfish remain later milestones.

### Tradeoffs

- Prefer a narrow but complete state path over broader shell architecture to land playability before M3 shell hardening.
- Prefer internal picking math over a generic dependency because the board is fixed and deterministic.
- Prefer procedural silhouettes with better lighting, highlights, and motion over authored assets so the milestone stays focused on interaction.
- Prefer a lightweight promotion and draw-claim HUD over deferring those flows and leaving some matches unfinishable.

## Milestones

### Milestone 1: Match session foundation and minimal state path

**Files**: crates/game_app/src/app.rs, crates/game_app/src/plugins/mod.rs, crates/game_app/src/plugins/app_shell.rs, crates/game_app/src/match_state.rs

**Flags**: bevy-state, domain-bridge

**Requirements**:

- Add a MatchSession resource that owns the live chess_core::GameState plus selection and pending-promotion state
- Wire MainMenu -> MatchLoading -> InMatch -> MatchResult transitions for a local match path
- Keep the existing shell scene and theme while adding a Start Local Match CTA and minimal rematch/return affordances

**Acceptance Criteria**:

- Starting a local match creates a fresh GameState::starting_position inside game_app
- The app can enter InMatch and MatchResult without inventing Bevy-local chess rules
- Minimal shell transitions support start, rematch, and return-to-menu flows required for M2

**Tests**:

- integration: game_app app-state smoke test for MainMenu -> MatchLoading -> InMatch
- integration: game_app rematch/result transition smoke test

#### Code Intent

- **CI-M-001-001** `crates/game_app/src/match_state.rs::MatchSession and interaction resources`: Introduce MatchSession, selected-square state, pending-promotion state, and helper methods that expose the current GameState, legal targets, and result status to Bevy systems without duplicating domain logic. (refs: DL-001, DL-005, DL-006)
- **CI-M-001-002** `crates/game_app/src/app.rs::build_app`: Register the new match-state resources and concrete M2 plugins, keeping the existing window/theme setup while enabling the local-match screen flow. (refs: DL-001, DL-004)
- **CI-M-001-003** `crates/game_app/src/plugins/mod.rs::plugin exports`: Replace scaffold-only exports for M2-owned plugins with concrete input and move-feedback modules while preserving later milestone seams for save/load, AI, and audio. (refs: DL-004)
- **CI-M-001-004** `crates/game_app/src/plugins/app_shell.rs::shell menus and screen transitions`: Extend the shell UI with a Start Local Match CTA, a lightweight MatchLoading handoff, and a MatchResult overlay that can rematch or return to the main menu without taking on M3-level shell scope. (refs: DL-004, DL-007)

#### Code Changes

**CC-M-001-001** (crates/game_app/src/match_state.rs) - implements CI-M-001-001

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/match_state.rs
@@
+use bevy::prelude::*;
+use chess_core::{DrawAvailability, GameState, GameStatus, Move, PieceKind, Square};
+
+#[derive(Resource)]
+pub struct MatchSession {
+    pub game_state: GameState,
+    pub selected_square: Option<Square>,
+    pub pending_promotion_move: Option<Move>,
+}
+
+impl MatchSession {
+    pub fn start_local_match() -> Self { /* initialize starting position */ }
+    pub fn legal_targets_for_selected(&self) -> Vec<Square> { /* derive from game_state */ }
+    pub fn status(&self) -> GameStatus { /* delegate to chess_core */ }
+    pub fn claimable_draw(&self) -> DrawAvailability { /* expose draw availability */ }
+}

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/src/match_state.rs
@@
+// MatchSession is the sole Bevy-facing bridge to chess_core during M2 so presentation never becomes the rules authority. (ref: DL-001)
+// Pending promotion and claimable-draw state live here because they are interaction concerns layered on top of chess_core legality. (ref: DL-005) (ref: DL-006)

```

> **Developer notes**: DL-001: centralize domain access so render/input systems consume one resource instead of re-deriving state.

**CC-M-001-002** (crates/game_app/src/app.rs) - implements CI-M-001-002

**Code:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@
-use crate::plugins::{
-    AiMatchPlugin, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin, MenuPlugin,
-    MoveFeedbackPlugin, PieceViewPlugin, SaveLoadPlugin, ShellInputPlugin,
-};
+use crate::match_state::MatchSession;
+use crate::plugins::{
+    AiMatchPlugin, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin, MenuPlugin,
+    MoveFeedbackPlugin, PieceViewPlugin, SaveLoadPlugin, ShellInputPlugin,
+};
@@
-        .init_state::<AppScreenState>()
+        .init_state::<AppScreenState>()
+        .insert_resource(MatchSession::start_local_match())
         .add_plugins((
             AppShellPlugin,
             BoardScenePlugin,
             PieceViewPlugin,
             ShellInputPlugin,
             MoveFeedbackPlugin,

```

**Documentation:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@
+// M2 adds a local-match resource and concrete interaction plugins while keeping the Bevy app responsible only for orchestration and presentation. (ref: DL-001) (ref: DL-004)

```

> **Developer notes**: DL-004: activate only the state path needed for a playable local match.

**CC-M-001-003** (crates/game_app/src/plugins/mod.rs) - implements CI-M-001-003

**Code:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@
 mod app_shell;
 mod board_scene;
+mod input;
+mod move_feedback;
 mod piece_view;
 mod scaffold;
@@
 pub use board_scene::BoardScenePlugin;
+pub use input::ShellInputPlugin;
+pub use move_feedback::MoveFeedbackPlugin;
 pub use piece_view::PieceViewPlugin;
 pub use scaffold::{
-    AiMatchPlugin, ChessAudioPlugin, MenuPlugin, MoveFeedbackPlugin, SaveLoadPlugin,
-    ShellInputPlugin,
+    AiMatchPlugin, ChessAudioPlugin, MenuPlugin, SaveLoadPlugin,
 };

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@
+// M2 promotes input and move-feedback from empty seams to concrete plugins; later milestone seams remain scaffolded. (ref: DL-004)

```

> **Developer notes**: Keep M2 plugin activation explicit without collapsing future milestone boundaries.

**CC-M-001-004** (crates/game_app/src/plugins/app_shell.rs) - implements CI-M-001-004

**Code:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@
-            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_shell_ui)
+            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_shell_ui)
+            .add_systems(OnEnter(AppScreenState::MatchLoading), initialize_local_match)
+            .add_systems(OnEnter(AppScreenState::MatchResult), spawn_match_result_ui)
@@
-                        Text::new("Rust workspace + Bevy shell baseline"),
+                        Text::new("Start Local Match"),
@@
+fn initialize_local_match(/* commands/resources */) { /* reset MatchSession then enter InMatch */ }
+fn spawn_match_result_ui(/* commands/resources */) { /* rematch + menu affordances */ }

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@
+// M2 keeps shell transitions intentionally narrow: menu -> load -> match -> result. Broader shell hardening remains M3 scope. (ref: DL-004)

```

> **Developer notes**: DL-004: do the smallest shell extension that still yields a complete local loop.

### Milestone 2: Domain-driven board presentation and square interaction

**Files**: crates/game_app/src/board_coords.rs, crates/game_app/src/plugins/board_scene.rs, crates/game_app/src/plugins/piece_view.rs, crates/game_app/src/plugins/input.rs

**Flags**: render-sync, picking

**Requirements**:

- Replace hardcoded piece spawning with rendering synchronized from MatchSession.game_state
- Give board squares stable square identity and visual-state hooks for selection, legal targets, and check highlighting
- Implement cursor-to-square mapping and click-to-select / click-to-target flow using chess_core legal moves

**Acceptance Criteria**:

- The visible piece layout exactly matches GameState at startup and after legal moves, captures, castling, en passant, and promotion updates
- Mouse clicks resolve to stable board squares without adding a third-party picking dependency
- Selection and legal-target highlighting are always derived from chess_core legal move data

**Tests**:

- unit: board square <-> world coordinate roundtrip
- integration: render sync piece-count and square occupancy checks after move application

#### Code Intent

- **CI-M-002-001** `crates/game_app/src/board_coords.rs::square/world mapping helpers`: Introduce one canonical board-coordinate module that converts between chess_core::Square, board-local coordinates, and world-space positions so rendering and picking cannot drift. (refs: DL-002, DL-003)
- **CI-M-002-002** `crates/game_app/src/plugins/board_scene.rs::board entities and highlight surfaces`: Annotate square meshes with their logical Square and add visual-state hooks for base, selected, legal-target, and king-in-check styling while preserving the current board material direction. (refs: DL-002, DL-007)
- **CI-M-002-003** `crates/game_app/src/plugins/piece_view.rs::piece sync from MatchSession`: Spawn and update piece entities from MatchSession.game_state instead of hardcoded ranks, carrying enough component metadata to support selection visuals and move animation. (refs: DL-001, DL-002, DL-007)
- **CI-M-002-004** `crates/game_app/src/plugins/input.rs::cursor picking and move-intent pipeline`: Translate mouse input into board-square intents, select only side-to-move pieces, preview legal targets from chess_core, and drive move attempts or pending promotion state without local move legality duplication. (refs: DL-001, DL-003, DL-005)

#### Code Changes

**CC-M-002-001** (crates/game_app/src/board_coords.rs) - implements CI-M-002-001

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/board_coords.rs
@@
+use bevy::prelude::*;
+use chess_core::Square;
+
+pub fn square_to_board_translation(square: Square, square_size: f32, board_height: f32) -> Vec3 { /* canonical mapping */ }
+pub fn world_to_square(world: Vec3, square_size: f32) -> Option<Square> { /* inverse mapping */ }
+pub fn board_plane_intersection(/* camera, cursor */) -> Option<Vec3> { /* ray-plane hit */ }

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/src/board_coords.rs
@@
+// M2 centralizes square/world mapping so board rendering, piece placement, and cursor picking share one coordinate contract. (ref: DL-002) (ref: DL-003)

```

> **Developer notes**: RISK-001 mitigation: one coordinate source prevents render/pick drift.

**CC-M-002-002** (crates/game_app/src/plugins/board_scene.rs) - implements CI-M-002-002

**Code:**

```diff
--- a/crates/game_app/src/plugins/board_scene.rs
+++ b/crates/game_app/src/plugins/board_scene.rs
@@
+#[derive(Component)]
+pub struct BoardSquareVisual {
+    pub square: chess_core::Square,
+}
@@
-                    parent.spawn((
+                    parent.spawn((
+                        BoardSquareVisual { square },
                         Mesh3d(square_mesh.clone()),
                         MeshMaterial3d(material),
                         Transform::from_xyz(x, 0.0, z),
                     ));
+
+fn update_square_visual_state(/* selected/legal/check resources */) { /* tint or emissive updates */ }

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/board_scene.rs
+++ b/crates/game_app/src/plugins/board_scene.rs
@@
+// Board squares gain logical square identity and visual-state hooks in M2 so input and feedback can target exact chess squares. (ref: DL-002)

```

> **Developer notes**: DL-007: preserve existing board look; add feedback layers instead of replacing geometry.

**CC-M-002-003** (crates/game_app/src/plugins/piece_view.rs) - implements CI-M-002-003

**Code:**

```diff
--- a/crates/game_app/src/plugins/piece_view.rs
+++ b/crates/game_app/src/plugins/piece_view.rs
@@
-use chess_core::{PieceKind, Side};
+use chess_core::{Piece, PieceKind, Side, Square};
@@
-fn spawn_piece_silhouettes(
+fn sync_piece_silhouettes_from_match(
     mut commands: Commands,
+    match_session: Res<MatchSession>,
     mut meshes: ResMut<Assets<Mesh>>,
     mut materials: ResMut<Assets<StandardMaterial>>,
     theme: Res<ShellTheme>,
 ) {
-    // spawn hardcoded ranks
+    // despawn stale pieces and respawn from match_session.game_state.board()
 }

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/piece_view.rs
+++ b/crates/game_app/src/plugins/piece_view.rs
@@
+// M2 replaces the shell-only starting layout with GameState-driven piece sync so the visual board cannot drift from chess_core. (ref: DL-002)

```

> **Developer notes**: DL-002: prefer deterministic resync from GameState over incremental entity authority.

**CC-M-002-004** (crates/game_app/src/plugins/input.rs) - implements CI-M-002-004

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/input.rs
@@
+use bevy::prelude::*;
+
+pub struct ShellInputPlugin;
+
+impl Plugin for ShellInputPlugin {
+    fn build(&self, app: &mut App) {
+        app.add_systems(Update, (pick_square_under_cursor, handle_square_clicks, handle_keyboard_match_actions));
+    }
+}
+
+fn pick_square_under_cursor(/* camera + board_coords */) { /* derive hovered square */ }
+fn handle_square_clicks(/* MatchSession */) { /* selection, move attempt, pending promotion */ }
+fn handle_keyboard_match_actions(/* Escape/Q/R/B/N */) { /* cancel and promotion shortcuts */ }

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/input.rs
@@
+// Input resolves to chess squares first and only then to domain actions so legal previews and move execution always flow through chess_core. (ref: DL-001) (ref: DL-003)

```

> **Developer notes**: RISK-002 mitigation: selection and targets come from chess_core legal_moves only.

### Milestone 3: Playable loop feedback, promotion, and M2 verification

**Files**: crates/game_app/src/plugins/move_feedback.rs, crates/game_app/src/plugins/app_shell.rs, crates/game_app/tests/board_mapping.rs, crates/game_app/tests/local_match_flow.rs, crates/game_app/tests/promotion_flow.rs

**Flags**: hud, promotion, testing

**Requirements**:

- Show turn, check, claimable draw, and terminal result feedback inside the in-match UI
- Provide a promotion chooser and visible move feedback strong enough to complete a full local match
- Add Bevy integration coverage for coordinate mapping, local move loop, promotion, and result transitions

**Acceptance Criteria**:

- A full local match can be completed with mouse and keyboard only, including promotion and claimable-draw resolution
- Check, checkmate, stalemate, automatic draws, and claimable draws produce visible in-app feedback
- Bevy tests cover at least coordinate mapping, render sync after move, promotion pending resolution, and terminal result transition

**Tests**:

- unit: board_mapping roundtrip and edge rejection
- integration: local_match_flow covers start, select, move, result transition
- integration: promotion_flow covers pending promotion and chosen-piece application

#### Code Intent

- **CI-M-003-001** `crates/game_app/src/plugins/move_feedback.rs::in-match HUD and animation feedback`: Render the active side, check/checkmate/draw feedback, claim-draw CTA, and lightweight move animation/highlight feedback without expanding into M3-level settings or menus. (refs: DL-006, DL-007)
- **CI-M-003-002** `crates/game_app/src/plugins/app_shell.rs::promotion and result overlays`: Host a focused promotion chooser plus terminal result overlay actions that consume MatchSession pending-promotion and outcome state. (refs: DL-004, DL-005, DL-006)
- **CI-M-003-003** `crates/game_app/tests/board_mapping.rs::board coordinate tests`: Add pure or app-light tests that prove square/world mapping is stable at center, edge, and out-of-bounds coordinates. (refs: DL-003)
- **CI-M-003-004** `crates/game_app/tests/local_match_flow.rs::playable loop integration tests`: Add Bevy integration tests that validate start-to-match flow, move execution, promotion pending resolution, claimable draw action, and terminal result transition using forced GameState scenarios where necessary. (refs: DL-001, DL-005, DL-006)

#### Code Changes

**CC-M-003-001** (crates/game_app/src/plugins/move_feedback.rs) - implements CI-M-003-001

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/move_feedback.rs
@@
+use bevy::prelude::*;
+
+pub struct MoveFeedbackPlugin;
+
+impl Plugin for MoveFeedbackPlugin {
+    fn build(&self, app: &mut App) {
+        app.add_systems(Update, (sync_match_hud, animate_active_move, update_claim_draw_banner));
+    }
+}
+
+fn sync_match_hud(/* MatchSession */) { /* side to move, check, result */ }
+fn animate_active_move(/* piece entities */) { /* transform interpolation */ }
+fn update_claim_draw_banner(/* draw availability */) { /* CTA visibility */ }

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/move_feedback.rs
@@
+// M2 feedback stays match-focused: turn state, check/result messaging, move motion, and draw-claim visibility without expanding into full shell UX. (ref: DL-006) (ref: DL-007)

```

> **Developer notes**: DL-007: put polish into readability and motion while keeping assets procedural.

**CC-M-003-002** (crates/game_app/src/plugins/app_shell.rs) - implements CI-M-003-002

**Code:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@
+fn spawn_promotion_overlay(/* pending promotion state */) { /* choose Q/R/B/N */ }
+fn handle_promotion_selection(/* keyboard + button input */) { /* finalize move */ }
+fn update_match_result_overlay(/* terminal GameStatus */) { /* rematch/menu CTA */ }

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@
+// Promotion and terminal overlays live in the shell because they are UI state layered over MatchSession, not additional domain rules. (ref: DL-005)

```

> **Developer notes**: DL-005: do not auto-apply promotion; wait for explicit player choice.

**CC-M-003-003** (crates/game_app/tests/board_mapping.rs) - implements CI-M-003-003

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/board_mapping.rs
@@
+use game_app::board_coords::{square_to_board_translation, world_to_square};
+
+#[test]
+fn square_world_mapping_roundtrips_centers_and_rejects_out_of_bounds() {
+    // verify center/edge squares and off-board coordinates
+}

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/board_mapping.rs
@@
+// Board mapping tests lock the coordinate contract shared by rendering and picking. (ref: DL-003)

```

> **Developer notes**: RISK-001 mitigation: protect the shared coordinate contract with pure tests.

**CC-M-003-004** (crates/game_app/tests/local_match_flow.rs) - implements CI-M-003-004

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/local_match_flow.rs
@@
+use bevy::prelude::*;
+
+#[test]
+fn local_match_flow_covers_start_move_promotion_draw_claim_and_result_transition() {
+    // drive a Bevy app through MainMenu -> InMatch and forced match scenarios
+}

```

**Documentation:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/local_match_flow.rs
@@
+// M2 integration tests prove the local match is actually completable with the Bevy orchestration path, not just via chess_core unit tests. (ref: DL-001) (ref: DL-005) (ref: DL-006)

```

> **Developer notes**: Exercise the Bevy-domain wiring, not just chess_core legality.

## Execution Waves

- W-001: M-001
- W-002: M-002
- W-003: M-003
