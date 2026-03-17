# Plan

## Overview

game_app already supports headless style Bevy integration tests but agent driven interaction still depends on duplicated test bootstraps and direct MatchSession or resource mutation instead of a stable semantic control surface

**Approach**: Add an opt in in process automation seam built from semantic commands snapshots and a shared headless harness then layer an optional feature gated stdio transport over the same contract without changing the default GUI runtime

### Automation Surface Overview

[Diagram pending Technical Writer rendering: DIAG-001]

```text
AutomationCommand -> AutomationHarness -> AutomationPlugin
                   -> MenuAction / SaveLoadRequest / MatchSession
                   -> AutomationSnapshot
```

### Command Model

- `AutomationCommand` stays at the semantic shell and match level: navigation, save/load, settings, match actions, and confirmations.
- `AutomationHarness::try_submit` executes one command and returns a fresh `AutomationSnapshot`.
- `AutomationPlugin` owns the in-process queue so integration tests and the optional `stdio` adapter stay on one dispatch path.

### Snapshot Model

- `AutomationSnapshot` exposes screen state, modal shell state, player-visible match state, and persisted save metadata.
- Snapshot fields intentionally use FEN, legal targets, pending promotion, last move, and save summaries instead of ECS identifiers or render details.
- Recovery availability and shell status stay in the snapshot so external adapters do not need direct resource reads.

### Non-Goals

- Native desktop click and key automation as a primary interface.
- Network-first control surfaces before the in-process contract exists.
- Separate command models for local harness execution and optional transport adapters.

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Start with an opt in in process automation seam inside game_app | Existing tests already drive Bevy without a real window -> semantic in process control reuses proven seams and avoids pixel fragility -> make this the primary design |
| DL-002 | Build observations from MatchSession and shell resources rather than ECS internals | chess_core remains the rules authority and Bevy entities are projections -> player visible state lives in MatchSession ShellMenuState SaveLoadState and RecoveryBannerState -> snapshots should read those surfaces only |
| DL-003 | Share semantic handlers between automation UI buttons and raw input | Current tests and input paths still mutate MatchSession directly in multiple places -> duplicated legality and shell routing would drift under automation -> extract reusable semantic handlers and route all callers through them |
| DL-004 | Add a deterministic headless harness builder over the existing plugin graph | game_app tests duplicate test_app bootstrap_shell and enter_local_match helpers across files -> a shared harness keeps production plugin wiring and headless setup aligned -> use that as the automation foundation |
| DL-005 | Keep transport separate and feature gated with stdio JSON Lines as the first adapter | Primary consumer is a local agent or script -> stdio fits single process orchestration without network lifecycle cost -> keep transport thin and optional over the same command model |
| DL-006 | Use behavior focused integration tests as the primary verification surface | The automation seam crosses menu routing persistence and gameplay -> real Bevy App stepping with SessionStore catches contract drift better than mock heavy unit tests -> make integration coverage the acceptance bar |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| Use native desktop click and key automation as the primary interface | Window focus render timing and host environment drift make it less reliable than semantic in process control (ref: DL-001) |
| Start with TCP or WebSocket transport before an in process seam exists | Transport lifecycle and serialization cost arrive before the command model is proven inside the local app (ref: DL-005) |
| Keep extending per test helpers and direct MatchSession mutation | That preserves duplicated test bootstrap code and bypasses the semantic behavior path players use (ref: DL-003) |

### Constraints

- [doc-derived] Keep chess_core authoritative for rules legality and outcomes
- [doc-derived] Fit the existing Bevy plugin message and resource architecture around AppScreenState and MatchSession
- [doc-derived] Preserve coarse top level screen states and use shell resources for modal surfaces
- [task-derived] Start with semantic in process automation before any external transport
- [task-derived] Keep automation opt in so normal gameplay stays unchanged when no agent surface is active
- [task-derived] Do not make native pixel automation the primary strategy
- [default-derived] Prefer behavior focused integration tests with real Bevy App stepping and real SessionStore backing

### Known Risks

- **Automation dispatch can fork away from player behavior if input and shell code keep private direct state mutation paths**: Extract shared semantic helpers for move selection promotion navigation save slots and settings then route automation and UI callers through them
- **Snapshot shape can leak unstable ECS details or omit legality critical shell state**: Build snapshots only from MatchSession ShellMenuState SaveLoadState and RecoveryBannerState and lock the contract with integration assertions
- **Headless automation can drift from the shipped plugin graph if tests keep bespoke app assembly helpers**: Expose a shared headless builder in app.rs and move new automation coverage onto that entry point
- **Optional transport can grow into a parallel runtime surface that changes the default player startup path**: Keep transport in a separate feature gated binary that only serializes AutomationCommand and AutomationSnapshot over stdio

## Invisible Knowledge

### System

game_app already models menu persistence and gameplay through semantic messages and resources around coarse screen states so agent automation should enter through that architecture rather than around OS input or raw ECS edits

### Invariants

- MatchSession remains the Bevy facing bridge to chess_core and automation never becomes a second rules authority
- Automation commands stay at the same semantic level as menu save load and match actions rather than pixels or widget ids
- Automation snapshots expose only player visible shell and match state and never expose Bevy entity ids
- Headless harness construction reuses the shipped plugin and resource graph rather than bespoke test only composition
- Out of process transport remains optional and cannot own a separate command model from in process automation

### Tradeoffs

- Prefer extracted semantic helpers over the minimal approach of adding more direct test mutation helpers
- Prefer explicit Bevy App stepping and snapshot assertions over brittle pixel automation for acceptance coverage
- Prefer stdio as the first external adapter because it fits the local agent use case without standing network services

## Milestones

### Milestone 1: Automation Contract And Harness

**Files**: crates/game_app/src/automation.rs, crates/game_app/src/app.rs, crates/game_app/src/lib.rs, crates/game_app/tests/automation_harness.rs

**Flags**: automation, harness, opt-in

**Requirements**:

- Define a semantic command and snapshot contract
- Add a deterministic headless app builder that reuses the shipped plugin graph
- Expose an AutomationHarness API without changing cargo run -p game_app

**Acceptance Criteria**:

- Harness boots to MainMenu and captures an initial snapshot
- The public contract exposes only semantic shell and match state
- Default player startup behavior stays unchanged

**Tests**:

- integration: boot harness and snapshot MainMenu
- integration: headless builder preserves default runtime path

#### Code Intent

- **CI-M-001-001** `crates/game_app/src/automation.rs::AutomationCommand AutomationSnapshot AutomationHarness`: Define semantic automation commands snapshots and harness helpers that expose shell match save and settings state without pixels widget ids or ECS entities (refs: DL-001, DL-002, DL-004)
- **CI-M-001-002** `crates/game_app/src/app.rs::build_headless_app install_shell_resources install_shell_plugins`: Add deterministic app construction helpers that reuse the shipped resource and plugin wiring for headless automation runs while leaving the default windowed build_app path unchanged (refs: DL-001, DL-004, DL-006)
- **CI-M-001-003** `crates/game_app/src/lib.rs::public exports`: Expose the automation contract and harness entry points for tests and future transport code without forcing automation into normal gameplay startup (refs: DL-004, DL-005)
- **CI-M-001-004** `crates/game_app/tests/automation_harness.rs::harness boot coverage`: Verify the harness boots to MainMenu reads an initial snapshot and preserves the unchanged player startup path (refs: DL-002, DL-004, DL-006)

#### Code Changes

**CC-M-001-005** (crates/game_app/src/automation.rs) - implements CI-M-001-001

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/automation.rs
@@ -0,0 +1,276 @@
+use std::fmt::{Display, Formatter};
+use std::path::PathBuf;
+
+use bevy::prelude::{App, World};
+use chess_core::{DrawAvailability, GameStatus, Move, PieceKind, Square};
+use chess_persistence::{SavedSessionSummary, ShellSettings};
+
+#[cfg(feature = "automation-transport")]
+use serde::{Deserialize, Serialize};
+
+use crate::app::{build_headless_app, AppScreenState};
+use crate::match_state::{ClaimedDrawReason, MatchSession};
+use crate::plugins::app_shell_logic;
+use crate::plugins::{
+    MenuContext, MenuPanel, RecoveryBannerState, SaveLoadState, ShellMenuState,
+};
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationScreen {
+    #[default]
+    Boot,
+    MainMenu,
+    MatchLoading,
+    InMatch,
+    MatchResult,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationMenuPanel {
+    #[default]
+    Home,
+    Setup,
+    LoadList,
+    Settings,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationMenuContext {
+    #[default]
+    MainMenu,
+    InMatchOverlay,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationConfirmationKind {
+    AbandonMatch,
+    DeleteSave,
+    OverwriteSave,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationClaimedDrawReason {
+    ThreefoldRepetition,
+    FiftyMoveRule,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(tag = "type", content = "action", rename_all = "snake_case")
+)]
+pub enum AutomationCommand {
+    Snapshot,
+    Step { frames: u32 },
+    Navigation(AutomationNavigationAction),
+    Save(AutomationSaveAction),
+    Settings(AutomationSettingsAction),
+    Match(AutomationMatchAction),
+    Confirm(AutomationConfirmationKind),
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(rename_all = "snake_case")
+)]
+pub enum AutomationNavigationAction {
+    OpenSetup,
+    BackToSetup,
+    OpenLoadList,
+    OpenSettings,
+    StartNewMatch,
+    ResumeRecovery,
+    PauseMatch,
+    ResumeMatch,
+    ReturnToMenu,
+    Rematch,
+    CancelModal,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(tag = "type", rename_all = "snake_case")
+)]
+pub enum AutomationSaveAction {
+    RefreshIndex,
+    SaveManual { label: Option<String> },
+    SelectSlot { slot_id: String },
+    LoadSelected,
+    DeleteSelected,
+    OverwriteSelected,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(tag = "type", rename_all = "snake_case")
+)]
+pub enum AutomationSettingsAction {
+    CycleRecoveryPolicy,
+    ToggleDisplayMode,
+    ToggleConfirmation { kind: AutomationConfirmationKind },
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize),
+    serde(tag = "type", rename_all = "snake_case")
+)]
+pub enum AutomationMatchAction {
+    SelectSquare { square: Square },
+    SubmitMove {
+        from: Square,
+        to: Square,
+        promotion: Option<PieceKind>,
+    },
+    ChoosePromotion { piece: PieceKind },
+    ClearInteraction,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum AutomationError {
+    InvalidStepCount(u32),
+    SaveSelectionRequired,
+    PromotionUnavailable,
+    CommandIgnored(String),
+}
+
+pub type AutomationResult<T> = Result<T, AutomationError>;
+
+impl Display for AutomationError {
+    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
+        match self {
+            Self::InvalidStepCount(frames) => {
+                write!(
+                    formatter,
+                    "automation step count must be at least one frame, got {frames}"
+                )
+            }
+            Self::SaveSelectionRequired => {
+                formatter.write_str("automation command requires a selected save slot")
+            }
+            Self::PromotionUnavailable => {
+                formatter.write_str("automation command requires a pending promotion move")
+            }
+            Self::CommandIgnored(reason) => {
+                write!(formatter, "automation command had no effect: {reason}")
+            }
+        }
+    }
+}
+
+impl std::error::Error for AutomationError {}
+
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize)
+)]
+pub struct AutomationMenuSnapshot {
+    pub panel: AutomationMenuPanel,
+    pub context: AutomationMenuContext,
+    pub confirmation: Option<AutomationConfirmationKind>,
+    pub selected_save: Option<String>,
+    pub status_line: Option<String>,
+    pub shell_status: Option<String>,
+    pub recovery_available: bool,
+    pub recovery_label: Option<String>,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize)
+)]
+pub struct AutomationMatchSnapshot {
+    pub fen: String,
+    pub status: GameStatus,
+    pub selected_square: Option<Square>,
+    pub legal_targets: Vec<Square>,
+    pub pending_promotion: Option<Move>,
+    pub last_move: Option<Move>,
+    pub claimable_draw: DrawAvailability,
+    pub claimed_draw: Option<AutomationClaimedDrawReason>,
+    pub dirty_recovery: bool,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Default)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize)
+)]
+pub struct AutomationSaveSnapshot {
+    pub manual_saves: Vec<SavedSessionSummary>,
+    pub recovery: Option<SavedSessionSummary>,
+    pub settings: ShellSettings,
+    pub last_message: Option<String>,
+    pub last_error: Option<String>,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+#[cfg_attr(
+    feature = "automation-transport",
+    derive(Serialize, Deserialize)
+)]
+pub struct AutomationSnapshot {
+    pub screen: AutomationScreen,
+    pub menu: AutomationMenuSnapshot,
+    pub match_state: AutomationMatchSnapshot,
+    pub saves: AutomationSaveSnapshot,
+}
+
+impl Default for AutomationMatchSnapshot {
+    fn default() -> Self {
+        Self {
+            fen: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
+            status: GameStatus::InProgress,
+            selected_square: None,
+            legal_targets: Vec::new(),
+            pending_promotion: None,
+            last_move: None,
+            claimable_draw: DrawAvailability::default(),
+            claimed_draw: None,
+            dirty_recovery: false,
+        }
+    }
+}
+
+impl Default for AutomationSnapshot {
+    fn default() -> Self {
+        Self {
+            screen: AutomationScreen::Boot,
+            menu: AutomationMenuSnapshot::default(),
+            match_state: AutomationMatchSnapshot::default(),
+            saves: AutomationSaveSnapshot::default(),
+        }
+    }
+}
+
+#[derive(Debug)]
+pub struct AutomationHarness {
+    pub(crate) app: App,
+}
+
+impl AutomationHarness {
+    #[must_use]
+    pub fn new(save_root: Option<PathBuf>) -> Self {
+        // Headless automation reuses the production shell graph so plans can target the same semantic seams as players.
+        Self {
+            app: build_headless_app(save_root),
+        }
+    }
+
+    pub fn step(&mut self) {
+        self.app.update();
+    }
+
+    /// Boot transition requires two frames: frame 1 runs the `OnEnter(Boot)` schedule
+    /// which triggers the state transition to `MainMenu`, frame 2 runs `OnEnter(MainMenu)`
+    /// which initializes the shell resources that snapshots depend on.
+    pub fn boot_to_main_menu(&mut self) {
+        self.step();
+        self.step();
+    }
+
+    pub fn step_until(
+        &mut self,
+        predicate: impl Fn(&AutomationSnapshot) -> bool,
+        max_frames: u32,
+    ) -> AutomationSnapshot {
+        for _ in 0..max_frames {
+            self.step();
+            let snapshot = self.snapshot();
+            if predicate(&snapshot) {
+                return snapshot;
+            }
+        }
+        panic!(
+            "automation harness did not satisfy predicate within {max_frames} frames"
+        );
+    }
+
+    #[must_use]
+    pub fn snapshot(&self) -> AutomationSnapshot {
+        capture_snapshot(self.app.world())
+    }
+
+    #[must_use]
+    pub fn app(&self) -> &App {
+        &self.app
+    }
+
+    pub fn app_mut(&mut self) -> &mut App {
+        &mut self.app
+    }
+}
+
+pub(crate) fn capture_snapshot(world: &World) -> AutomationSnapshot {
+    let state = world.resource::<bevy::prelude::State<AppScreenState>>();
+    let menu_state = world.resource::<ShellMenuState>();
+    let save_state = world.resource::<SaveLoadState>();
+    let recovery = world.resource::<RecoveryBannerState>();
+    let match_session = world.resource::<MatchSession>();
+
+    AutomationSnapshot {
+        screen: screen_from_state(*state.get()),
+        menu: AutomationMenuSnapshot {
+            panel: panel_from_state(menu_state.panel),
+            context: context_from_state(menu_state.context),
+            confirmation: menu_state.confirmation.map(confirmation_from_state),
+            selected_save: menu_state.selected_save.clone(),
+            status_line: menu_state.status_line.clone(),
+            shell_status: app_shell_logic::effective_shell_status(
+                menu_state,
+                save_state,
+                recovery,
+            ),
+            recovery_available: recovery.available,
+            recovery_label: recovery.label.clone(),
+        },
+        match_state: AutomationMatchSnapshot {
+            fen: match_session.game_state().to_fen(),
+            status: match_session.status(),
+            selected_square: match_session.selected_square,
+            legal_targets: match_session.legal_targets_for_selected(),
+            pending_promotion: match_session.pending_promotion_move,
+            last_move: match_session.last_move,
+            claimable_draw: match_session.claimable_draw(),
+            claimed_draw: match_session.claimed_draw_reason().map(claimed_draw_from_state),
+            dirty_recovery: match_session.is_recovery_dirty(),
+        },
+        saves: AutomationSaveSnapshot {
+            manual_saves: save_state.manual_saves.clone(),
+            recovery: save_state.recovery.clone(),
+            settings: save_state.settings.clone(),
+            last_message: save_state.last_message.clone(),
+            last_error: save_state.last_error.clone(),
+        },
+    }
+}
+
+fn screen_from_state(state: AppScreenState) -> AutomationScreen {
+    match state {
+        AppScreenState::Boot => AutomationScreen::Boot,
+        AppScreenState::MainMenu => AutomationScreen::MainMenu,
+        AppScreenState::MatchLoading => AutomationScreen::MatchLoading,
+        AppScreenState::InMatch => AutomationScreen::InMatch,
+        AppScreenState::MatchResult => AutomationScreen::MatchResult,
+    }
+}
+
+fn panel_from_state(panel: MenuPanel) -> AutomationMenuPanel {
+    match panel {
+        MenuPanel::Home => AutomationMenuPanel::Home,
+        MenuPanel::Setup => AutomationMenuPanel::Setup,
+        MenuPanel::LoadList => AutomationMenuPanel::LoadList,
+        MenuPanel::Settings => AutomationMenuPanel::Settings,
+    }
+}
+
+fn context_from_state(context: MenuContext) -> AutomationMenuContext {
+    match context {
+        MenuContext::MainMenu => AutomationMenuContext::MainMenu,
+        MenuContext::InMatchOverlay => AutomationMenuContext::InMatchOverlay,
+    }
+}
+
+fn confirmation_from_state(
+    kind: crate::plugins::ConfirmationKind,
+) -> AutomationConfirmationKind {
+    match kind {
+        crate::plugins::ConfirmationKind::AbandonMatch => {
+            AutomationConfirmationKind::AbandonMatch
+        }
+        crate::plugins::ConfirmationKind::DeleteSave => AutomationConfirmationKind::DeleteSave,
+        crate::plugins::ConfirmationKind::OverwriteSave => {
+            AutomationConfirmationKind::OverwriteSave
+        }
+    }
+}
+
+fn claimed_draw_from_state(kind: ClaimedDrawReason) -> AutomationClaimedDrawReason {
+    match kind {
+        ClaimedDrawReason::ThreefoldRepetition => {
+            AutomationClaimedDrawReason::ThreefoldRepetition
+        }
+        ClaimedDrawReason::FiftyMoveRule => AutomationClaimedDrawReason::FiftyMoveRule,
+    }
+}
```

**Documentation:**

```diff
--- a/crates/game_app/src/automation.rs
+++ b/crates/game_app/src/automation.rs
@@ -1,3 +1,7 @@
+// Semantic automation contract for `game_app`.
+// Commands and snapshots stay at the shell and match boundary so agents and
+// optional adapters reuse player-visible semantics. (refs: DL-001, DL-002)
+
 use std::fmt::{Display, Formatter};
 use std::path::PathBuf;
 

```


**CC-M-001-006** (crates/game_app/src/app.rs) - implements CI-M-001-002

**Code:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@ -1,4 +1,7 @@
+use std::path::PathBuf;
+
+use bevy::state::app::StatesPlugin;
 use bevy::prelude::*;
 use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};

@@ -23,7 +26,7 @@ pub enum AppScreenState {
 pub fn build_app() -> App {
     let shell_theme = ShellTheme::default();
     let mut app = App::new();

-    install_shell_resources(&mut app, shell_theme)
+    install_shell_resources(&mut app, shell_theme, None)
         .add_plugins(DefaultPlugins.set(WindowPlugin {
             primary_window: Some(primary_window()),
             ..default()
@@ -35,6 +38,24 @@ pub fn build_app() -> App {

     app
 }
+
+#[must_use]
+pub fn build_headless_app(save_root: Option<PathBuf>) -> App {
+    let shell_theme = ShellTheme::default();
+    let mut app = App::new();
+
+    app.add_plugins(MinimalPlugins)
+        .add_plugins(StatesPlugin)
+        .insert_resource(Assets::<Mesh>::default())
+        .insert_resource(Assets::<StandardMaterial>::default())
+        .insert_resource(ButtonInput::<KeyCode>::default())
+        .insert_resource(ButtonInput::<MouseButton>::default());
+    // Headless automation keeps the production shell graph so tests and agents exercise the same menu and persistence seams.
+    install_shell_resources(&mut app, shell_theme, save_root).init_state::<AppScreenState>();
+    install_shell_plugins(&mut app);
+
+    app
+}

 pub fn run() {
     build_app().run();
@@ -49,7 +70,11 @@ fn primary_window() -> Window {
     ..default()
 }

-fn install_shell_resources(app: &mut App, shell_theme: ShellTheme) -> &mut App {
+fn install_shell_resources(
+    app: &mut App,
+    shell_theme: ShellTheme,
+    save_root: Option<PathBuf>,
+) -> &mut App {
     app.insert_resource(ClearColor(shell_theme.clear_color))
         .insert_resource(shell_theme)
         .insert_resource(MatchSession::start_local_match())
@@ -59,7 +84,7 @@ fn install_shell_resources(app: &mut App, shell_theme: ShellTheme) -> &mut App {
         .insert_resource(ShellMenuState::default())
         .insert_resource(RecoveryBannerState::default())
         .insert_resource(SaveLoadState::default())
-        .insert_resource(SaveRootOverride::default())
+        .insert_resource(SaveRootOverride(save_root))
 }

 fn install_shell_plugins(app: &mut App) -> &mut App {
@@ -95,7 +120,7 @@ mod tests {

         let mut app = App::new();
         app.add_plugins((MinimalPlugins, StatesPlugin));
         app.init_state::<AppScreenState>();
-        install_shell_resources(&mut app, ShellTheme::default());
+        install_shell_resources(&mut app, ShellTheme::default(), None);

         assert_eq!(
             app.world().resource::<State<AppScreenState>>().get(),
```

**Documentation:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@ -1,3 +1,7 @@
+// Headless app construction reuses the shipped shell graph so automation and
+// integration tests exercise the same plugins and resources as player startup.
+// (refs: DL-004, DL-006)
+
 use bevy::prelude::*;
 use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
 

```


**CC-M-001-007** (crates/game_app/src/lib.rs) - implements CI-M-001-003

**Code:**

```diff
--- a/crates/game_app/src/lib.rs
+++ b/crates/game_app/src/lib.rs
@@ -1,9 +1,29 @@
 pub mod board_coords;
+mod automation;
+
+#[cfg(feature = "automation-transport")]
+pub mod automation_transport;

 mod app;
 mod match_state;
 mod plugins;
 mod style;

-pub use app::{APP_TITLE, AppScreenState, build_app, run};
+pub use app::{APP_TITLE, AppScreenState, build_app, build_headless_app, run};
+pub use automation::{
+    AutomationClaimedDrawReason, AutomationCommand, AutomationConfirmationKind,
+    AutomationError, AutomationHarness, AutomationMatchAction, AutomationMatchSnapshot,
+    AutomationMenuContext, AutomationMenuPanel, AutomationMenuSnapshot,
+    AutomationNavigationAction, AutomationResult, AutomationSaveAction,
+    AutomationSaveSnapshot, AutomationScreen, AutomationSettingsAction,
+    AutomationSnapshot,
+};
 pub use match_state::{
     ClaimedDrawReason, MatchLaunchIntent, MatchSession, MatchSessionSummary, PendingLoadedSnapshot,
 };
```

**Documentation:**

```diff
--- a/crates/game_app/src/lib.rs
+++ b/crates/game_app/src/lib.rs
@@ -1,3 +1,7 @@
+// Automation re-exports stay separate from `run` so harness consumers can opt
+// into the contract without changing the GUI entry point.
+// (refs: DL-004, DL-005)
+
 pub mod board_coords;
 
 mod app;

```


**CC-M-001-008** (crates/game_app/tests/automation_harness.rs) - implements CI-M-001-004

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/automation_harness.rs
@@ -0,0 +1,38 @@
+use tempfile::tempdir;
+
+use bevy::prelude::*;
+use game_app::{
+    AppScreenState, AutomationHarness, AutomationMenuPanel, AutomationScreen,
+    SaveRootOverride, build_app,
+};
+
+#[test]
+fn harness_boots_to_main_menu_and_reads_initial_snapshot() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut harness = AutomationHarness::new(Some(root.path().to_path_buf()));
+
+    harness.boot_to_main_menu();
+    let snapshot = harness.snapshot();
+
+    assert_eq!(snapshot.screen, AutomationScreen::MainMenu);
+    assert_eq!(snapshot.menu.panel, AutomationMenuPanel::Home);
+    assert!(snapshot.saves.manual_saves.is_empty());
+    assert_eq!(snapshot.menu.selected_save, None);
+}
+
+#[test]
+fn windowed_builder_keeps_default_shell_startup_contract() {
+    let app = build_app();
+
+    assert_eq!(
+        app.world().resource::<State<AppScreenState>>().get(),
+        &AppScreenState::Boot
+    );
+    assert_eq!(
+        app.world().resource::<SaveRootOverride>(),
+        &SaveRootOverride::default()
+    );
+}
```

**Documentation:**

```diff
--- a/crates/game_app/tests/automation_harness.rs
+++ b/crates/game_app/tests/automation_harness.rs
@@ -1,3 +1,7 @@
+// Integration coverage for the headless harness contract.
+// These assertions keep the automation seam aligned with the shipped shell
+// startup path. (refs: DL-004, DL-006)
+
 use tempfile::tempdir;
 
 use bevy::prelude::*;

```


### Milestone 2: Semantic Command Routing

**Files**: crates/game_app/src/match_state.rs, crates/game_app/src/plugins/automation.rs, crates/game_app/src/plugins/mod.rs, crates/game_app/src/plugins/input.rs, crates/game_app/src/plugins/app_shell.rs, crates/game_app/src/plugins/save_load.rs, crates/game_app/tests/automation_semantic_flow.rs

**Flags**: semantic-routing, integration-tests, opt-in

**Requirements**:

- Route automation commands through menu save load and match semantics
- Extract shared handlers where input or shell code mutates state directly
- Refresh snapshots with enough data for agent reasoning after each update

**Acceptance Criteria**:

- Automation starts resumes rematches saves loads and returns to menu without direct ECS mutation
- Automation drives legal moves and promotion through shared semantic handlers
- Snapshots expose FEN selection legal targets shell status and recovery state

**Tests**:

- integration: start match and play legal moves via automation
- integration: promotion save load and recovery flows via automation

#### Code Intent

- **CI-M-002-001** `crates/game_app/src/plugins/automation.rs::AutomationPlugin command dispatch snapshot refresh`: Own the automation command queue dispatch semantic commands through shell and match seams and refresh a snapshot resource after each update (refs: DL-001, DL-002, DL-003)
- **CI-M-002-002** `crates/game_app/src/plugins/input.rs::shared match action helpers`: Extract square selection move submission promotion choice and interaction clearing helpers so raw input and automation reuse one legality path (refs: DL-003, DL-006)
- **CI-M-002-003** `crates/game_app/src/plugins/app_shell.rs::shared shell action helpers`: Route automation and UI button flows through the same navigation save slot confirmation and promotion helpers instead of parallel shell logic (refs: DL-003, DL-005)
- **CI-M-002-004** `crates/game_app/src/plugins/save_load.rs::semantic save and settings requests`: Represent save load recovery refresh and settings operations as semantic requests that automation can reuse without direct SaveLoadState mutation (refs: DL-003, DL-006)
- **CI-M-002-005** `crates/game_app/src/match_state.rs::automation match snapshot helpers`: Provide stable helpers for FEN selected square legal targets pending promotion draw availability last move and recovery dirtiness so snapshots stay player visible and domain aligned (refs: DL-002, DL-003)
- **CI-M-002-006** `crates/game_app/src/plugins/mod.rs::plugin wiring exports`: Wire the automation plugin and exports into opt in harness composition without changing the default player plugin graph (refs: DL-004, DL-005)
- **CI-M-002-007** `crates/game_app/tests/automation_semantic_flow.rs::end to end semantic coverage`: Exercise start move promotion save load recovery rematch and return to menu flows through automation commands and snapshot assertions (refs: DL-001, DL-002, DL-006)

#### Code Changes

**CC-M-002-001** (crates/game_app/src/plugins/automation.rs) - implements CI-M-002-001

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/automation.rs
@@ -0,0 +1,141 @@
+use std::collections::VecDeque;
+
+use bevy::prelude::*;
+
+use crate::app::AppScreenState;
+use crate::automation::{
+    capture_snapshot, AutomationCommand, AutomationError, AutomationHarness,
+    AutomationResult, AutomationSnapshot,
+};
+use crate::match_state::MatchSession;
+
+use super::app_shell::{
+    handle_confirmation_action, handle_navigation_action, handle_save_slot_action,
+    handle_settings_action,
+};
+use super::input::apply_match_action;
+use super::menu::{MenuAction, ShellMenuState};
+use super::save_load::{SaveLoadRequest, SaveLoadState};
+
+#[derive(Resource, Default)]
+struct AutomationCommandQueue(VecDeque<AutomationCommand>);
+
+#[derive(Resource, Debug, Clone, Default)]
+pub struct AutomationSnapshotResource(pub AutomationSnapshot);
+
+#[derive(Resource, Debug, Clone, Default)]
+struct AutomationLastError(Option<AutomationError>);
+
+pub struct AutomationPlugin;
+
+impl Plugin for AutomationPlugin {
+    fn build(&self, app: &mut App) {
+        app.init_resource::<AutomationCommandQueue>()
+            .init_resource::<AutomationSnapshotResource>()
+            .init_resource::<AutomationLastError>()
+            .add_systems(
+                Update,
+                (dispatch_automation_commands, refresh_automation_snapshot).chain(),
+            );
+    }
+}
+
+fn dispatch_automation_commands(
+    mut queue: ResMut<AutomationCommandQueue>,
+    mut last_error: ResMut<AutomationLastError>,
+    state: Res<State<AppScreenState>>,
+    menu_state: Res<ShellMenuState>,
+    save_state: Res<SaveLoadState>,
+    mut menu_actions: MessageWriter<MenuAction>,
+    mut save_requests: MessageWriter<SaveLoadRequest>,
+    mut match_session: ResMut<MatchSession>,
+) {
+    last_error.0 = None;
+    while let Some(command) = queue.0.pop_front() {
+        let result = match command {
+            AutomationCommand::Navigation(action) => {
+                handle_navigation_action(
+                    action,
+                    *state.get(),
+                    menu_state.as_ref(),
+                    save_state.as_ref(),
+                    &mut menu_actions,
+                    &mut save_requests,
+                );
+                Ok(())
+            }
+            AutomationCommand::Save(action) => handle_save_slot_action(
+                &action,
+                menu_state.as_ref(),
+                save_state.as_ref(),
+                &mut menu_actions,
+                &mut save_requests,
+                match_session.as_ref(),
+            ),
+            AutomationCommand::Settings(action) => {
+                handle_settings_action(&action, &mut save_requests);
+                Ok(())
+            }
+            AutomationCommand::Match(action) => {
+                apply_match_action(match_session.as_mut(), &action)
+            }
+            AutomationCommand::Confirm(kind) => handle_confirmation_action(
+                kind,
+                menu_state.as_ref(),
+                save_state.as_ref(),
+                &mut menu_actions,
+                &mut save_requests,
+            ),
+            AutomationCommand::Snapshot | AutomationCommand::Step { .. } => Ok(()),
+        };
+
+        if let Err(error) = result {
+            last_error.0 = Some(error);
+            break;
+        }
+    }
+}
+
+fn refresh_automation_snapshot(world: &mut World) {
+    let snapshot = capture_snapshot(world);
+    world.resource_mut::<AutomationSnapshotResource>().0 = snapshot;
+}
+
+impl AutomationHarness {
+    #[must_use]
+    pub fn with_semantic_automation(mut self) -> Self {
+        self.ensure_semantic_automation();
+        self
+    }
+
+    pub fn try_submit(&mut self, command: AutomationCommand) -> AutomationResult<AutomationSnapshot> {
+        self.ensure_semantic_automation();
+        match command {
+            AutomationCommand::Snapshot => {}
+            AutomationCommand::Step { frames } => {
+                if frames == 0 {
+                    return Err(AutomationError::InvalidStepCount(frames));
+                }
+                for _ in 0..frames {
+                    self.app.update();
+                }
+            }
+            command => {
+                self.app.world_mut().resource_mut::<AutomationCommandQueue>().0.push_back(command);
+                // Frame 1: dispatch_automation_commands runs and writes MenuAction / SaveLoadRequest messages.
+                // Frame 2: downstream systems (save_load, menu routing) observe those messages and apply state changes.
+                self.app.update();
+                self.app.update();
+                if let Some(error) = self.app.world_mut().resource_mut::<AutomationLastError>().0.take() {
+                    return Err(error);
+                }
+            }
+        }
+
+        Ok(self.snapshot())
+    }
+
+    fn ensure_semantic_automation(&mut self) {
+        if self.app.world().contains_resource::<AutomationCommandQueue>() {
+            return;
+        }
+
+        self.app.add_plugins(AutomationPlugin);
+        let snapshot = capture_snapshot(self.app.world());
+        self.app.world_mut().resource_mut::<AutomationSnapshotResource>().0 = snapshot;
+    }
+}
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/automation.rs
+++ b/crates/game_app/src/plugins/automation.rs
@@ -1,3 +1,7 @@
+// Semantic automation dispatch lives in a dedicated plugin so commands flow
+// through the same shell and match handlers instead of a parallel path.
+// (refs: DL-001, DL-003)
+
 use std::collections::VecDeque;
 
 use bevy::prelude::*;

```


**CC-M-002-002** (crates/game_app/src/plugins/input.rs) - implements CI-M-002-002

**Code:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -1,10 +1,11 @@
 use bevy::prelude::*;
 use bevy::window::PrimaryWindow;
-use chess_core::{Move, PieceKind};
+use chess_core::{Move, PieceKind, Square};

 use super::menu::{MenuAction, MenuContext, ShellMenuState};
 use super::save_load::SaveLoadRequest;
 use crate::app::AppScreenState;
+use crate::automation::{AutomationError, AutomationMatchAction, AutomationResult};
 use crate::board_coords::{board_plane_intersection, world_to_square};
 use crate::match_state::MatchSession;
 use crate::style::ShellTheme;
@@ -71,57 +72,7 @@ fn handle_square_clicks(
         return;
     }

-    let Some(clicked_square) = hovered_square.0 else {
-        match_session.selected_square = None;
-        match_session.mark_recovery_dirty();
-        return;
-    };
-    if match_session.pending_promotion_move.is_some() {
-        return;
-    }
-
-    let current_side = match_session.game_state().side_to_move();
-    let clicked_piece = match_session.piece_at(clicked_square);
-
-    let Some(selected_square) = match_session.selected_square else {
-        if clicked_piece.is_some_and(|piece| piece.side == current_side) {
-            match_session.selected_square = Some(clicked_square);
-            match_session.mark_recovery_dirty();
-        }
-        return;
-    };
-
-    if clicked_square == selected_square {
-        match_session.clear_interaction();
-        match_session.mark_recovery_dirty();
-        return;
-    }
-
-    if clicked_piece.is_some_and(|piece| piece.side == current_side) {
-        match_session.selected_square = Some(clicked_square);
-        match_session.mark_recovery_dirty();
-        return;
-    }
-
-    let candidate_moves: Vec<_> = match_session
-        .game_state()
-        .legal_moves()
-        .into_iter()
-        .filter(|candidate| candidate.from() == selected_square && candidate.to() == clicked_square)
-        .collect();
-
-    if candidate_moves.is_empty() {
-        match_session.selected_square = None;
-        match_session.mark_recovery_dirty();
-        return;
-    }
-
-    if candidate_moves
-        .iter()
-        .any(|candidate| candidate.promotion().is_some())
-    {
-        match_session.pending_promotion_move = Some(Move::new(selected_square, clicked_square));
-        match_session.mark_recovery_dirty();
-        return;
-    }
-
-    let _ = match_session.apply_move(candidate_moves[0]);
+    apply_square_interaction(match_session.as_mut(), hovered_square.0);
 }

 fn handle_keyboard_match_actions(
@@ -141,8 +92,7 @@ fn handle_keyboard_match_actions(
         } else if match_session.pending_promotion_move.is_some()
             || match_session.selected_square.is_some()
         {
-            match_session.clear_interaction();
-            match_session.mark_recovery_dirty();
+            clear_match_interaction(match_session.as_mut());
         } else {
             menu_actions.write(MenuAction::PauseMatch);
         }
@@ -176,9 +126,79 @@ fn handle_keyboard_match_actions(
     None
 };

-    if let Some(promotion_kind) = promotion_kind {
-        let _ = match_session.apply_move(Move::with_promotion(
-            pending_move.from(),
-            pending_move.to(),
-            promotion_kind,
-        ));
+    if let Some(promotion_kind) = promotion_kind {
+        let _ = apply_promotion_choice(match_session.as_mut(), promotion_kind);
+    }
+}
+
+pub(crate) fn apply_match_action(
+    match_session: &mut MatchSession,
+    action: &AutomationMatchAction,
+) -> AutomationResult<()> {
+    match action {
+        AutomationMatchAction::SelectSquare { square } => {
+            apply_square_interaction(match_session, Some(*square));
+            Ok(())
+        }
+        AutomationMatchAction::SubmitMove {
+            from,
+            to,
+            promotion,
+        } => {
+            apply_square_interaction(match_session, Some(*from));
+            apply_square_interaction(match_session, Some(*to));
+            if let Some(piece) = promotion {
+                apply_promotion_choice(match_session, *piece)?;
+            }
+            Ok(())
+        }
+        AutomationMatchAction::ChoosePromotion { piece } => {
+            apply_promotion_choice(match_session, *piece)
+        }
+        AutomationMatchAction::ClearInteraction => {
+            clear_match_interaction(match_session);
+            Ok(())
+        }
+    }
+}
+
+pub(crate) fn apply_square_interaction(
+    match_session: &mut MatchSession,
+    clicked_square: Option<Square>,
+) {
+    let Some(clicked_square) = clicked_square else {
+        clear_match_interaction(match_session);
+        return;
+    };
+    if match_session.pending_promotion_move.is_some() {
+        return;
+    }
+
+    let current_side = match_session.game_state().side_to_move();
+    let clicked_piece = match_session.piece_at(clicked_square);
+
+    let Some(selected_square) = match_session.selected_square else {
+        if clicked_piece.is_some_and(|piece| piece.side == current_side) {
+            match_session.selected_square = Some(clicked_square);
+            match_session.mark_recovery_dirty();
+        }
+        return;
+    };
+
+    if clicked_square == selected_square {
+        clear_match_interaction(match_session);
+        return;
+    }
+
+    if clicked_piece.is_some_and(|piece| piece.side == current_side) {
+        match_session.selected_square = Some(clicked_square);
+        match_session.mark_recovery_dirty();
+        return;
+    }
+
+    let candidate_moves: Vec<_> = match_session
+        .game_state()
+        .legal_moves()
+        .into_iter()
+        .filter(|candidate| {
+            candidate.from() == selected_square && candidate.to() == clicked_square
+        })
+        .collect();
+
+    if candidate_moves.is_empty() {
+        clear_match_interaction(match_session);
+        return;
+    }
+
+    if candidate_moves.iter().any(|candidate| candidate.promotion().is_some()) {
+        match_session.pending_promotion_move = Some(Move::new(selected_square, clicked_square));
+        match_session.mark_recovery_dirty();
+        return;
 }
+
+    let _ = match_session.apply_move(candidate_moves[0]);
+}
+
+pub(crate) fn clear_match_interaction(match_session: &mut MatchSession) {
+    match_session.clear_interaction();
+    match_session.mark_recovery_dirty();
+}
+
+pub(crate) fn apply_promotion_choice(
+    match_session: &mut MatchSession,
+    promotion_kind: PieceKind,
+) -> AutomationResult<()> {
+    let Some(pending_move) = match_session.pending_promotion_move else {
+        return Err(AutomationError::PromotionUnavailable);
+    };
+    let _ = match_session.apply_move(Move::with_promotion(
+        pending_move.from(),
+        pending_move.to(),
+        promotion_kind,
+    ));
+    Ok(())
 }

 fn overlay_captures_match_input(menu_state: &ShellMenuState) -> bool {
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -1,3 +1,7 @@
+// Shared match interaction helpers keep raw input and automation on one
+// legality path while `MatchSession` stays authoritative.
+// (refs: DL-003, DL-006)
+
 use bevy::prelude::*;
 use bevy::window::PrimaryWindow;
 use chess_core::{Move, PieceKind};

```


**CC-M-002-003** (crates/game_app/src/plugins/app_shell.rs) - implements CI-M-002-003

**Code:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
            +++ b/crates/game_app/src/plugins/app_shell.rs
            @@ -4,9 +4,13 @@
             use bevy::prelude::*;
             use chess_core::PieceKind;
             use chess_persistence::DisplayMode;

            +use crate::automation::{
            +    AutomationConfirmationKind, AutomationError, AutomationNavigationAction,
            +    AutomationResult, AutomationSaveAction, AutomationSettingsAction,
            +};
             use super::app_shell_logic;
             use super::menu::{
                 ConfirmationKind, MenuAction, MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState,
             };
            +use super::input::apply_promotion_choice;
             use super::save_load::{SaveLoadRequest, SaveLoadState};
             use crate::app::AppScreenState;
             use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
            @@ -74,6 +78,20 @@ enum ShellAction {
                 Promote(PieceKind),
             }

            +impl From<ConfirmationKind> for AutomationConfirmationKind {
            +    fn from(value: ConfirmationKind) -> Self {
            +        match value {
            +            ConfirmationKind::AbandonMatch => Self::AbandonMatch,
            +            ConfirmationKind::DeleteSave => Self::DeleteSave,
            +            ConfirmationKind::OverwriteSave => Self::OverwriteSave,
            +        }
            +    }
            +}
            +
            +impl From<AutomationConfirmationKind> for ConfirmationKind {
            +    fn from(value: AutomationConfirmationKind) -> Self {
            +        match value {
            +            AutomationConfirmationKind::AbandonMatch => Self::AbandonMatch,
            +            AutomationConfirmationKind::DeleteSave => Self::DeleteSave,
            +            AutomationConfirmationKind::OverwriteSave => Self::OverwriteSave,
            +        }
            +    }
            +}
            +
            +fn navigation_from_shell(action: &ShellAction) -> Option<AutomationNavigationAction> {
            +    match action {
            +        ShellAction::OpenSetup => Some(AutomationNavigationAction::OpenSetup),
            +        ShellAction::BackToSetup => Some(AutomationNavigationAction::BackToSetup),
            +        ShellAction::StartNewMatch => Some(AutomationNavigationAction::StartNewMatch),
            +        ShellAction::OpenLoadList => Some(AutomationNavigationAction::OpenLoadList),
            +        ShellAction::OpenSettings => Some(AutomationNavigationAction::OpenSettings),
            +        ShellAction::ResumeRecovery => Some(AutomationNavigationAction::ResumeRecovery),
            +        ShellAction::ResumeMatch => Some(AutomationNavigationAction::ResumeMatch),
            +        ShellAction::ReturnToMenu => Some(AutomationNavigationAction::ReturnToMenu),
            +        ShellAction::Rematch => Some(AutomationNavigationAction::Rematch),
            +        ShellAction::CancelModal => Some(AutomationNavigationAction::CancelModal),
            +        _ => None,
            +    }
            +}
            +
            +fn save_from_shell(action: &ShellAction) -> Option<AutomationSaveAction> {
            +    match action {
            +        ShellAction::SaveManual => Some(AutomationSaveAction::SaveManual { label: None }),
            +        ShellAction::OverwriteSelectedSave => Some(AutomationSaveAction::OverwriteSelected),
            +        ShellAction::LoadSelected => Some(AutomationSaveAction::LoadSelected),
            +        ShellAction::DeleteSelected => Some(AutomationSaveAction::DeleteSelected),
            +        ShellAction::SelectSave(slot_id) => Some(AutomationSaveAction::SelectSlot {
            +            slot_id: slot_id.clone(),
            +        }),
            +        _ => None,
            +    }
            +}
            +
            +fn settings_from_shell(action: &ShellAction) -> Option<AutomationSettingsAction> {
            +    match action {
            +        ShellAction::CycleRecoveryPolicy => Some(AutomationSettingsAction::CycleRecoveryPolicy),
            +        ShellAction::ToggleDisplayMode => Some(AutomationSettingsAction::ToggleDisplayMode),
            +        ShellAction::ToggleConfirmation(kind) => Some(AutomationSettingsAction::ToggleConfirmation {
            +            kind: (*kind).into(),
            +        }),
            +        _ => None,
            +    }
            +}
            +
             #[derive(Component)]
             struct ShellCamera {
                 orbit_angle: f32,
            @@ -841,7 +859,7 @@ fn handle_shell_button_actions(
                 interaction_query: Query<(&Interaction, &ShellActionButton), Changed<Interaction>>,
                 state: Res<State<AppScreenState>>,
                 menu_state: Res<ShellMenuState>,
            -    mut save_state: ResMut<SaveLoadState>,
            +    save_state: Res<SaveLoadState>,
                 mut menu_actions: MessageWriter<MenuAction>,
                 mut save_requests: MessageWriter<SaveLoadRequest>,
                 mut match_session_mut: ResMut<MatchSession>,
            @@ -852,51 +870,67 @@ fn handle_shell_button_actions(
                         continue;
                     }

                     match &button_action.action {
            +            action if navigation_from_shell(action).is_some() => {
            +                handle_navigation_action(
            +                    navigation_from_shell(action).unwrap(),
            +                    *state.get(),
            +                    menu_state.as_ref(),
            +                    save_state.as_ref(),
            +                    &mut menu_actions,
            +                    &mut save_requests,
            +                );
            +            }
            +            action if save_from_shell(action).is_some() => {
            +                let _ = handle_save_slot_action(
            +                    &save_from_shell(action).unwrap(),
            +                    menu_state.as_ref(),
            +                    save_state.as_ref(),
            +                    &mut menu_actions,
            +                    &mut save_requests,
            +                    match_session_mut.as_ref(),
            +                );
            +            }
            +            action if settings_from_shell(action).is_some() => {
            +                handle_settings_action(
            +                    &settings_from_shell(action).unwrap(),
            +                    &mut save_requests,
            +                );
            +            }
            +            ShellAction::Confirm(kind) => {
            +                let _ = handle_confirmation_action(
            +                    (*kind).into(),
            +                    menu_state.as_ref(),
            +                    save_state.as_ref(),
            +                    &mut menu_actions,
            +                    &mut save_requests,
            +                );
            +            }
                         ShellAction::Promote(piece_kind) => {
            +                let _ = handle_promotion_action(*piece_kind, match_session_mut.as_mut());
                         }
                     }
                 }
             }

            -fn handle_navigation_action(
            -    action: &ShellAction,
            +pub(crate) fn handle_navigation_action(
            +    action: AutomationNavigationAction,
                 state: AppScreenState,
                 menu_state: &ShellMenuState,
                 save_state: &SaveLoadState,
                 menu_actions: &mut MessageWriter<MenuAction>,
                 save_requests: &mut MessageWriter<SaveLoadRequest>,
             ) {
                 match action {
            -        ShellAction::OpenSetup => {
            +        AutomationNavigationAction::OpenSetup => {
                         menu_actions.write(MenuAction::OpenSetup);
                     }
            -        ShellAction::BackToSetup => {
            +        AutomationNavigationAction::BackToSetup => {
                         menu_actions.write(MenuAction::BackToSetup);
                     }
            -        ShellAction::StartNewMatch => {
            +        AutomationNavigationAction::OpenLoadList => {
            +            menu_actions.write(MenuAction::OpenLoadList);
            +        }
            +        AutomationNavigationAction::OpenSettings => {
            +            menu_actions.write(MenuAction::OpenSettings);
            +        }
            +        AutomationNavigationAction::StartNewMatch => {
                         menu_actions.write(MenuAction::StartNewMatch);
                     }
            -        ShellAction::OpenLoadList => {
            -            menu_actions.write(MenuAction::OpenLoadList);
            +        AutomationNavigationAction::ResumeRecovery => {
            +            save_requests.write(SaveLoadRequest::ResumeRecovery);
                     }
            -        ShellAction::OpenSettings => {
            -            menu_actions.write(MenuAction::OpenSettings);
            +        AutomationNavigationAction::PauseMatch => {
            +            menu_actions.write(MenuAction::PauseMatch);
                     }
            -        ShellAction::ResumeRecovery => {
            -            save_requests.write(SaveLoadRequest::ResumeRecovery);
            -        }
            -        ShellAction::ResumeMatch => {
            +        AutomationNavigationAction::ResumeMatch => {
                         menu_actions.write(MenuAction::ResumeMatch);
                     }
            -        ShellAction::ReturnToMenu => {
            +        AutomationNavigationAction::ReturnToMenu => {
                         request_return_to_menu(state, menu_state, save_state, menu_actions, save_requests);
                     }
            -        ShellAction::Rematch => {
            +        AutomationNavigationAction::Rematch => {
                         menu_actions.write(MenuAction::Rematch);
                     }
            -        _ => {}
            +        AutomationNavigationAction::CancelModal => {
            +            menu_actions.write(MenuAction::CancelModal);
            +        }
                     }
                 }

            -fn handle_save_slot_action(
            -    action: &ShellAction,
            +pub(crate) fn handle_save_slot_action(
            +    action: &AutomationSaveAction,
                 menu_state: &ShellMenuState,
                 save_state: &SaveLoadState,
                 menu_actions: &mut MessageWriter<MenuAction>,
                 save_requests: &mut MessageWriter<SaveLoadRequest>,
                 match_session: &MatchSession,
            -) {
            +) -> AutomationResult<()> {
                 match action {
            -        ShellAction::SaveManual => {
            +        AutomationSaveAction::RefreshIndex => {
            +            save_requests.write(SaveLoadRequest::RefreshIndex);
            +        }
            +        AutomationSaveAction::SaveManual { label } => {
                         save_requests.write(SaveLoadRequest::SaveManual {
            -                label: app_shell_logic::derive_save_label(match_session.last_move),
            +                label: label
            +                    .clone()
            +                    .unwrap_or_else(|| app_shell_logic::derive_save_label(match_session.last_move)),
                             slot_id: None,
                         });
                     }
            -        ShellAction::OverwriteSelectedSave => {
            +        AutomationSaveAction::OverwriteSelected => {
                         if let Some(selected) = app_shell_logic::selected_save_summary(menu_state, save_state) {
                             if save_state.settings.confirm_actions.overwrite_save {
                                 menu_actions.write(MenuAction::RequestConfirmation(
                                     ConfirmationKind::OverwriteSave,
                                 ));
                             } else {
                                 save_requests.write(SaveLoadRequest::SaveManual {
                                     label: selected.label.clone(),
                                     slot_id: Some(selected.slot_id.clone()),
                                 });
                             }
            +            } else {
            +                return Err(AutomationError::SaveSelectionRequired);
                         }
                     }
            -        ShellAction::LoadSelected => {
            +        AutomationSaveAction::LoadSelected => {
                         if let Some(slot_id) = menu_state.selected_save.clone() {
                             save_requests.write(SaveLoadRequest::LoadManual { slot_id });
            +            } else {
            +                return Err(AutomationError::SaveSelectionRequired);
                         }
                     }
            -        ShellAction::DeleteSelected => {
            +        AutomationSaveAction::DeleteSelected => {
                         if let Some(slot_id) = menu_state.selected_save.clone() {
                             if save_state.settings.confirm_actions.delete_save {
                                 menu_actions.write(MenuAction::RequestConfirmation(
                                     ConfirmationKind::DeleteSave,
                                 ));
                             } else {
                                 save_requests.write(SaveLoadRequest::DeleteManual { slot_id });
                             }
            +            } else {
            +                return Err(AutomationError::SaveSelectionRequired);
                         }
                     }
            -        ShellAction::SelectSave(slot_id) => {
            +        AutomationSaveAction::SelectSlot { slot_id } => {
                         menu_actions.write(MenuAction::SelectSave(slot_id.clone()));
                     }
            -        _ => {}
                 }
+            Ok(())
             }

            -fn handle_settings_action(
            -    action: &ShellAction,
            -    save_state: &mut SaveLoadState,
            +pub(crate) fn handle_settings_action(
            +    action: &AutomationSettingsAction,
                 save_requests: &mut MessageWriter<SaveLoadRequest>,
             ) {
                 match action {
            -        ShellAction::CycleRecoveryPolicy => {
            -            save_state.settings.recovery_policy =
            -                app_shell_logic::next_recovery_policy(save_state.settings.recovery_policy);
            -            save_requests.write(SaveLoadRequest::PersistSettings);
            +        AutomationSettingsAction::CycleRecoveryPolicy => {
            +            save_requests.write(SaveLoadRequest::CycleRecoveryPolicy);
                     }
            -        ShellAction::ToggleDisplayMode => {
            -            save_state.settings.display_mode = match save_state.settings.display_mode {
            -                DisplayMode::Windowed => DisplayMode::Fullscreen,
            -                DisplayMode::Fullscreen => DisplayMode::Windowed,
            -            };
            -            save_requests.write(SaveLoadRequest::PersistSettings);
            +        AutomationSettingsAction::ToggleDisplayMode => {
            +            save_requests.write(SaveLoadRequest::ToggleDisplayMode);
                     }
            -        ShellAction::ToggleConfirmation(kind) => {
            -            match kind {
            -                ConfirmationKind::AbandonMatch => {
            -                    save_state.settings.confirm_actions.abandon_match =
            -                        !save_state.settings.confirm_actions.abandon_match;
            -                }
            -                ConfirmationKind::DeleteSave => {
            -                    save_state.settings.confirm_actions.delete_save =
            -                        !save_state.settings.confirm_actions.delete_save;
            -                }
            -                ConfirmationKind::OverwriteSave => {
            -                    save_state.settings.confirm_actions.overwrite_save =
            -                        !save_state.settings.confirm_actions.overwrite_save;
            -                }
            -            }
            -            save_requests.write(SaveLoadRequest::PersistSettings);
            -        }
            -        _ => {}
            +        AutomationSettingsAction::ToggleConfirmation { kind } => {
            +            save_requests.write(SaveLoadRequest::ToggleConfirmation((*kind).into()));
            +        }
                 }
             }

            -fn handle_confirmation_action(
            -    action: &ShellAction,
            +pub(crate) fn handle_confirmation_action(
            +    kind: AutomationConfirmationKind,
                 menu_state: &ShellMenuState,
                 save_state: &SaveLoadState,
                 menu_actions: &mut MessageWriter<MenuAction>,
                 save_requests: &mut MessageWriter<SaveLoadRequest>,
            -) {
            -    match action {
            -        ShellAction::CancelModal => {
            -            menu_actions.write(MenuAction::CancelModal);
            -        }
            -        ShellAction::Confirm(kind) => match kind {
            -            ConfirmationKind::AbandonMatch => {
            +) -> AutomationResult<()> {
            +    match ConfirmationKind::from(kind) {
            +        ConfirmationKind::AbandonMatch => {
                         save_requests.write(SaveLoadRequest::AbandonMatchAndReturnToMenu);
                     }
            -            ConfirmationKind::DeleteSave => {
            +        ConfirmationKind::DeleteSave => {
                         if let Some(slot_id) = menu_state.selected_save.clone() {
                             save_requests.write(SaveLoadRequest::DeleteManual { slot_id });
            +            } else {
            +                return Err(AutomationError::SaveSelectionRequired);
                         }
                         menu_actions.write(MenuAction::CancelModal);
                     }
            -            ConfirmationKind::OverwriteSave => {
            +        ConfirmationKind::OverwriteSave => {
                         if let Some(selected) =
                             app_shell_logic::selected_save_summary(menu_state, save_state)
                         {
                             save_requests.write(SaveLoadRequest::SaveManual {
                                 label: selected.label.clone(),
                                 slot_id: Some(selected.slot_id.clone()),
                             });
            +            } else {
            +                return Err(AutomationError::SaveSelectionRequired);
                         }
                         menu_actions.write(MenuAction::CancelModal);
            -            }
            -        },
            -        _ => {}
            +        }
                 }
+            Ok(())
             }

            -fn handle_promotion_action(piece_kind: PieceKind, match_session: &mut MatchSession) {
            -    if let Some(pending_move) = match_session.pending_promotion_move {
            -        let _ = match_session.apply_move(chess_core::Move::with_promotion(
            -            pending_move.from(),
            -            pending_move.to(),
            -            piece_kind,
            -        ));
            -    }
            +fn handle_promotion_action(
            +    piece_kind: PieceKind,
            +    match_session: &mut MatchSession,
            +) -> AutomationResult<()> {
            +    apply_promotion_choice(match_session, piece_kind)
             }
            @@ -1245,10 +1279,10 @@ mod tests {
                     let (mut menu_actions, mut save_requests) = writers.get_mut(&mut world);
                         handle_navigation_action(
            -                &ShellAction::OpenSetup,
            +                AutomationNavigationAction::OpenSetup,
                             AppScreenState::MainMenu,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                         );
                         handle_navigation_action(
            -                &ShellAction::ResumeRecovery,
            +                AutomationNavigationAction::ResumeRecovery,
                             AppScreenState::MainMenu,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                         );
                         handle_navigation_action(
            -                &ShellAction::ReturnToMenu,
            +                AutomationNavigationAction::ReturnToMenu,
                             AppScreenState::InMatch,
                             &menu_state,
                             &save_state,
            @@ -1304,43 +1338,50 @@ mod tests {
                     let (mut menu_actions, mut save_requests) = writers.get_mut(&mut world);
                         handle_save_slot_action(
            -                &ShellAction::SaveManual,
            +                &AutomationSaveAction::SaveManual { label: None },
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                             &match_session,
            -            );
            +            )
            +            .expect("save manual action should be routable");
                         handle_save_slot_action(
            -                &ShellAction::OverwriteSelectedSave,
            +                &AutomationSaveAction::OverwriteSelected,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                             &match_session,
            -            );
            +            )
            +            .expect("overwrite action should be routable");
                         handle_save_slot_action(
            -                &ShellAction::DeleteSelected,
            +                &AutomationSaveAction::DeleteSelected,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                             &match_session,
            -            );
            +            )
            +            .expect("delete action should be routable");
                         handle_save_slot_action(
            -                &ShellAction::LoadSelected,
            +                &AutomationSaveAction::LoadSelected,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
                             &match_session,
            -            );
            +            )
            +            .expect("load action should be routable");
                         handle_confirmation_action(
            -                &ShellAction::Confirm(ConfirmationKind::DeleteSave),
            +                AutomationConfirmationKind::DeleteSave,
                             &menu_state,
                             &save_state,
                             &mut menu_actions,
                             &mut save_requests,
            -            );
            +            )
            +            .expect("confirmation action should be routable");
                     }

                     let menu_messages = drain_messages::<MenuAction>(&mut world);
                     let save_messages = drain_messages::<SaveLoadRequest>(&mut world);
            @@ -1369,32 +1410,26 @@ mod tests {
                     let mut world = World::new();
                     world.init_resource::<Messages<SaveLoadRequest>>();
                     let mut writers: SaveWriterState<'_, '_> = SystemState::new(&mut world);

            -        let mut save_state = SaveLoadState {
            -            settings: ShellSettings {
            -                recovery_policy: RecoveryStartupPolicy::Ask,
            -                confirm_actions: ConfirmActionSettings::default(),
            -                display_mode: DisplayMode::Windowed,
            -            },
            -            ..Default::default()
            -        };
                         {
                             let (mut save_requests,) = writers.get_mut(&mut world);
                             handle_settings_action(
            -                &ShellAction::CycleRecoveryPolicy,
            -                &mut save_state,
            +                &AutomationSettingsAction::CycleRecoveryPolicy,
                                 &mut save_requests,
                             );
                             handle_settings_action(
            -                &ShellAction::ToggleDisplayMode,
            -                &mut save_state,
            +                &AutomationSettingsAction::ToggleDisplayMode,
                                 &mut save_requests,
                             );
                             handle_settings_action(
            -                &ShellAction::ToggleConfirmation(ConfirmationKind::DeleteSave),
            -                &mut save_state,
            +                &AutomationSettingsAction::ToggleConfirmation {
            +                    kind: AutomationConfirmationKind::DeleteSave,
            +                },
                                 &mut save_requests,
                             );
                         }
                         let save_messages = drain_messages::<SaveLoadRequest>(&mut world);
            -        assert_eq!(
            -            save_state.settings.recovery_policy,
            -            RecoveryStartupPolicy::Ignore
            -        );
            -        assert_eq!(save_state.settings.display_mode, DisplayMode::Fullscreen);
            -        assert!(!save_state.settings.confirm_actions.delete_save);
            -        assert_eq!(save_messages.len(), 3);
            +        assert!(save_messages.contains(&SaveLoadRequest::CycleRecoveryPolicy));
            +        assert!(save_messages.contains(&SaveLoadRequest::ToggleDisplayMode));
            +        assert!(save_messages.contains(&SaveLoadRequest::ToggleConfirmation(
            +            ConfirmationKind::DeleteSave
            +        )));

                     type LaunchState<'w, 's> = SystemState<(
                         ResMut<'w, MatchSession>,
            @@ -1513,7 +1548,9 @@ mod tests {
                     };
                     match_session.replace_game_state(promotion_ready);
                     match_session.pending_promotion_move = Some(Move::new(from, to));
            -        handle_promotion_action(PieceKind::Queen, &mut match_session);
            +        handle_promotion_action(PieceKind::Queen, &mut match_session)
            +            .expect("promotion action should resolve the pending move");
                     assert_eq!(match_session.pending_promotion_move, None);
                 }
             }
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@ -1,3 +1,7 @@
+// Shell handlers translate semantic navigation, save, settings, and
+// confirmation actions into the existing message flow.
+// (refs: DL-003, DL-005)
+
 //! Presentation layer for the coarse app shell.
 //! Main menu, pause overlay, and results render from modal resources while match launch still funnels through MatchLoading. (ref: DL-001) (ref: DL-007)
 

```


**CC-M-002-004** (crates/game_app/src/plugins/save_load.rs) - implements CI-M-002-004

**Code:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
            +++ b/crates/game_app/src/plugins/save_load.rs
            @@ -9,7 +9,8 @@ use chess_persistence::{
                 SnapshotMetadata, StoreResult,
             };

            -use super::menu::{MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState};
            +use super::app_shell_logic;
            +use super::menu::{ConfirmationKind, MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState};
             use super::save_load_logic;
             use crate::app::AppScreenState;
             use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
            @@ -63,6 +64,9 @@ pub enum SaveLoadRequest {
                 ClearRecovery,
                 AbandonMatchAndReturnToMenu,
                 PersistSettings,
            +    CycleRecoveryPolicy,
            +    ToggleDisplayMode,
            +    ToggleConfirmation(ConfirmationKind),
             }

             #[derive(Resource, Default)]
            @@ -275,24 +279,28 @@ fn handle_save_load_requests(
                             "Unable to clear interrupted-session recovery.",
                         ));
                     }
            +        SaveLoadRequest::CycleRecoveryPolicy => {
            +            save_state.settings.recovery_policy =
            +                app_shell_logic::next_recovery_policy(save_state.settings.recovery_policy);
            +            persist_settings_update(&store, &mut save_state, &mut recovery_banner);
            +        }
            +        SaveLoadRequest::ToggleDisplayMode => {
            +            save_state.settings.display_mode = match save_state.settings.display_mode {
            +                DisplayMode::Windowed => DisplayMode::Fullscreen,
            +                DisplayMode::Fullscreen => DisplayMode::Windowed,
            +            };
            +            persist_settings_update(&store, &mut save_state, &mut recovery_banner);
            +        }
            +        SaveLoadRequest::ToggleConfirmation(kind) => {
            +            match kind {
            +                ConfirmationKind::AbandonMatch => {
            +                    save_state.settings.confirm_actions.abandon_match =
            +                        !save_state.settings.confirm_actions.abandon_match;
            +                }
            +                ConfirmationKind::DeleteSave => {
            +                    save_state.settings.confirm_actions.delete_save =
            +                        !save_state.settings.confirm_actions.delete_save;
            +                }
            +                ConfirmationKind::OverwriteSave => {
            +                    save_state.settings.confirm_actions.overwrite_save =
            +                        !save_state.settings.confirm_actions.overwrite_save;
            +                }
            +            }
            +            persist_settings_update(&store, &mut save_state, &mut recovery_banner);
            +        }
                     SaveLoadRequest::PersistSettings => {
+                    persist_settings_update(&store, &mut save_state, &mut recovery_banner);
+                }
                 }
             }
            @@ -361,6 +369,22 @@ fn clear_result_recovery_cache(
                     }
                 }

+fn persist_settings_update(
+    store: &SessionStoreResource,
+    save_state: &mut SaveLoadState,
+    recovery_banner: &mut RecoveryBannerState,
+) {
+    match store.0.save_settings(&save_state.settings) {
+        Ok(()) => {
+            save_state.last_error = None;
+            save_state.last_message = Some(String::from("Saved shell settings."));
+            save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
+        }
+        Err(_) => {
+            save_state.last_error = Some(String::from("Unable to save shell settings."));
+            save_load_logic::sync_cached_recovery_visibility(save_state, recovery_banner);
+        }
+    }
+}
+
             fn refresh_store_index_from_resource(
                 store: &SessionStoreResource,
                 save_state: &mut SaveLoadState,
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -1,3 +1,7 @@
+// Save and settings requests remain centralized here so shell and automation
+// send semantic intents instead of mutating durable state directly.
+// (refs: DL-003, DL-006)
+
 //! Shell persistence orchestration for manual saves, interrupted-session recovery, and settings.
 //! Repository I/O lives here so manual saves, interrupted-session recovery, and the shipped settings trio of startup recovery, destructive confirmations, and display mode stay behind one snapshot-based boundary. (ref: DL-002) (ref: DL-005) (ref: DL-007) (ref: DL-008)
 //! Extracted helpers carry branch-heavy copy and recovery-visibility rules so the Bevy plugin remains in scope while direct tests cover the decision surface. (ref: DL-002) (ref: DL-004) (ref: DL-007)

```


**CC-M-002-005** (crates/game_app/src/match_state.rs) - implements CI-M-002-005

**Code:**

```diff
--- a/crates/game_app/src/match_state.rs
            +++ b/crates/game_app/src/match_state.rs
            @@ -126,6 +126,36 @@ impl MatchSession {
                 pub fn game_state(&self) -> &GameState {
                     &self.game_state
                 }
            +
            +    #[must_use]
            +    pub fn fen(&self) -> String {
            +        self.game_state.to_fen()
            +    }
            +
            +    #[must_use]
            +    pub fn selected_square(&self) -> Option<Square> {
            +        self.selected_square
            +    }
            +
            +    #[must_use]
            +    pub fn legal_targets(&self) -> Vec<Square> {
            +        self.legal_targets_for_selected()
            +    }
            +
            +    #[must_use]
            +    pub fn pending_promotion(&self) -> Option<Move> {
            +        self.pending_promotion_move
            +    }
            +
            +    #[must_use]
            +    pub fn last_move_played(&self) -> Option<Move> {
            +        self.last_move
            +    }
            +
            +    #[must_use]
            +    pub fn draw_availability(&self) -> DrawAvailability {
            +        self.claimable_draw()
            +    }
            +
            +    #[must_use]
            +    pub fn claimed_draw_state(&self) -> Option<ClaimedDrawReason> {
            +        self.claimed_draw
            +    }

                 pub fn replace_game_state(&mut self, game_state: GameState) {
                     self.game_state = game_state;
            @@ -303,6 +333,19 @@ mod tests {
                     assert!(session.summary().dirty_recovery);
                 }

+    #[test]
+    fn stable_snapshot_helpers_surface_player_visible_state() {
+        let session = MatchSession::restore_from_snapshot(&sample_snapshot(true));
+        assert_eq!(session.fen(), "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1");
+        assert_eq!(session.selected_square(), Some(square("e7")));
+        assert_eq!(session.legal_targets(), vec![square("e8")]);
+        assert_eq!(session.pending_promotion(), Some(Move::new(square("e7"), square("e8"))));
+        assert_eq!(session.last_move_played(), Some(Move::new(square("e7"), square("e8"))));
+        assert_eq!(
+            session.claimed_draw_state(),
+            Some(ClaimedDrawReason::ThreefoldRepetition)
+        );
+        assert!(!session.draw_availability().is_claimable());
+    }
             }
```

**Documentation:**

```diff
--- a/crates/game_app/src/match_state.rs
+++ b/crates/game_app/src/match_state.rs
@@ -1,3 +1,7 @@
+// Snapshot accessors expose player-visible match state without leaking ECS
+// internals or moving chess authority out of `MatchSession`.
+// (ref: DL-002)
+
 //! Bevy-facing match bridge for local play, load, and recovery flows.
 //! Snapshot conversion keeps `chess_core` authoritative while the shell restores only the interaction state it needs. (ref: DL-001) (ref: DL-004)
 

```


**CC-M-002-006** (crates/game_app/src/plugins/mod.rs) - implements CI-M-002-006

**Code:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,3 +1,4 @@
+mod automation;
 mod app_shell;
 pub mod app_shell_logic;
 mod board_scene;
@@ -8,6 +9,7 @@ pub mod save_load_logic;
 mod scaffold;

+pub use automation::AutomationPlugin;
 pub use app_shell::AppShellPlugin;
 pub use board_scene::{BoardScenePlugin, BoardSquareVisual};
 pub use input::ShellInputPlugin;
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,3 +1,6 @@
+// Automation plugin exports remain opt-in so headless harness composition does
+// not alter the default player plugin graph. (refs: DL-004, DL-005)
+
 mod app_shell;
 pub mod app_shell_logic;
 mod board_scene;

```


**CC-M-002-007** (crates/game_app/tests/automation_semantic_flow.rs) - implements CI-M-002-007

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/automation_semantic_flow.rs
@@ -0,0 +1,151 @@
+use tempfile::tempdir;
+
+use chess_core::{GameState, Move, PieceKind, Square};
+use chess_persistence::{GameSnapshot, SaveKind, SessionStore, SnapshotMetadata, SnapshotShellState};
+use game_app::{
+    AutomationCommand, AutomationConfirmationKind, AutomationHarness,
+    AutomationMatchAction, AutomationNavigationAction, AutomationSaveAction,
+    AutomationScreen,
+};
+
+fn square(name: &str) -> Square {
+    Square::from_algebraic(name).expect("test square must be valid")
+}
+
+fn manual_snapshot(label: &str, fen: &str) -> GameSnapshot {
+    GameSnapshot::from_parts(
+        GameState::from_fen(fen).expect("fixture FEN should parse"),
+        SnapshotMetadata {
+            label: label.to_string(),
+            created_at_utc: Some(String::from("2026-03-17T00:00:00Z")),
+            updated_at_utc: None,
+            notes: Some(String::from("automation fixture")),
+            save_kind: SaveKind::Manual,
+            session_id: label.to_ascii_lowercase().replace(' ', "-"),
+            recovery_key: None,
+        },
+        SnapshotShellState::default(),
+    )
+}
+
+fn recovery_snapshot(label: &str, fen: &str) -> GameSnapshot {
+    GameSnapshot::from_parts(
+        GameState::from_fen(fen).expect("fixture FEN should parse"),
+        SnapshotMetadata {
+            label: label.to_string(),
+            created_at_utc: Some(String::from("2026-03-17T00:00:00Z")),
+            updated_at_utc: None,
+            notes: Some(String::from("recovery fixture")),
+            save_kind: SaveKind::Recovery,
+            session_id: String::from("recovery"),
+            recovery_key: Some(String::from("autosave")),
+        },
+        SnapshotShellState::default(),
+    )
+}
+
+#[test]
+fn automation_commands_cover_start_move_save_rematch_and_return_to_menu() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut harness =
+        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
+    harness.boot_to_main_menu();
+
+    harness
+        .try_submit(AutomationCommand::Navigation(
+            AutomationNavigationAction::StartNewMatch,
+        ))
+        .expect("start command should route through automation");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 3 })
+        .expect("match loading should settle");
+    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
+
+    let snapshot = harness
+        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
+            from: square("e2"),
+            to: square("e4"),
+            promotion: None,
+        }))
+        .expect("semantic move should reuse the same legality path");
+    assert_eq!(snapshot.match_state.last_move, Some(Move::new(square("e2"), square("e4"))));
+
+    harness
+        .try_submit(AutomationCommand::Save(AutomationSaveAction::SaveManual {
+            label: Some(String::from("Automation Save")),
+        }))
+        .expect("manual save should be routable");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 2 })
+        .expect("save refresh should settle");
+    assert_eq!(snapshot.saves.manual_saves.len(), 1);
+
+    harness
+        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
+            slot_id: snapshot.saves.manual_saves[0].slot_id.clone(),
+        }))
+        .expect("slot selection should be routable");
+    harness
+        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
+        .expect("load should be routable");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 3 })
+        .expect("loaded match should settle");
+    assert_eq!(snapshot.match_state.last_move, Some(Move::new(square("e2"), square("e4"))));
+
+    harness
+        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::Rematch))
+        .expect("rematch should be routable");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 3 })
+        .expect("rematch should settle");
+    assert_eq!(snapshot.match_state.last_move, None);
+
+    harness
+        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::PauseMatch))
+        .expect("pause should be routable");
+    harness
+        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::ReturnToMenu))
+        .expect("return to menu should be routable");
+    harness
+        .try_submit(AutomationCommand::Confirm(AutomationConfirmationKind::AbandonMatch))
+        .expect("abandon confirmation should be routable");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 2 })
+        .expect("menu transition should settle");
+    assert_eq!(snapshot.screen, AutomationScreen::MainMenu);
+}
+
+#[test]
+fn automation_commands_cover_load_and_promotion_choice() {
+    let root = tempdir().expect("temporary directory should be created");
+    let summary = SessionStore::new(root.path())
+        .save_manual(manual_snapshot("Promotion Fixture", "7k/P7/8/8/8/8/8/4K3 w - - 0 1"))
+        .expect("fixture save should succeed");
+    let mut harness =
+        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
+    harness.boot_to_main_menu();
+
+    harness
+        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::OpenSetup))
+        .expect("setup should be routable");
+    harness
+        .try_submit(AutomationCommand::Navigation(AutomationNavigationAction::OpenLoadList))
+        .expect("load list should be routable");
+    harness
+        .try_submit(AutomationCommand::Save(AutomationSaveAction::SelectSlot {
+            slot_id: summary.slot_id.clone(),
+        }))
+        .expect("slot selection should be routable");
+    harness
+        .try_submit(AutomationCommand::Save(AutomationSaveAction::LoadSelected))
+        .expect("load should be routable");
+    harness
+        .try_submit(AutomationCommand::Step { frames: 3 })
+        .expect("loaded promotion fixture should settle");
+
+    let snapshot = harness
+        .try_submit(AutomationCommand::Match(AutomationMatchAction::SubmitMove {
+            from: square("a7"),
+            to: square("a8"),
+            promotion: None,
+        }))
+        .expect("promotion setup move should be routable");
+    assert_eq!(snapshot.match_state.pending_promotion, Some(Move::new(square("a7"), square("a8"))));
+
+    let snapshot = harness
+        .try_submit(AutomationCommand::Match(AutomationMatchAction::ChoosePromotion {
+            piece: PieceKind::Queen,
+        }))
+        .expect("promotion choice should be routable");
+    assert_eq!(
+        snapshot.match_state.last_move,
+        Some(Move::with_promotion(square("a7"), square("a8"), PieceKind::Queen))
+    );
+}
+
+#[test]
+fn automation_commands_cover_recovery_resume() {
+    let root = tempdir().expect("temporary directory should be created");
+    SessionStore::new(root.path())
+        .store_recovery(recovery_snapshot("Recovery Fixture", "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1"))
+        .expect("recovery fixture should succeed");
+    let mut harness =
+        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
+    harness.boot_to_main_menu();
+
+    let snapshot = harness.snapshot();
+    assert!(snapshot.menu.recovery_available);
+    let snapshot = harness
+        .try_submit(AutomationCommand::Navigation(
+            AutomationNavigationAction::ResumeRecovery,
+        ))
+        .expect("recovery resume should be routable");
+    let snapshot = harness
+        .try_submit(AutomationCommand::Step { frames: 3 })
+        .expect("recovery resume should settle");
+    assert_eq!(snapshot.screen, AutomationScreen::InMatch);
+    assert_eq!(snapshot.match_state.fen, "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");
+}
```

**Documentation:**

```diff
--- a/crates/game_app/tests/automation_semantic_flow.rs
+++ b/crates/game_app/tests/automation_semantic_flow.rs
@@ -1,3 +1,7 @@
+// End-to-end automation coverage drives gameplay, save/load, and recovery
+// through semantic commands instead of direct resource mutation.
+// (refs: DL-001, DL-006)
+
 use tempfile::tempdir;
 
 use chess_core::{GameState, Move, PieceKind, Square};

```


### Milestone 3: Optional Transport And Documentation

**Files**: crates/game_app/Cargo.toml, crates/game_app/src/automation_transport.rs, crates/game_app/src/bin/game_app_agent.rs, crates/game_app/tests/automation_transport.rs, plans/agent-testing.md, README.md, crates/game_app/README.md

**Flags**: transport, feature-gated, docs

**Requirements**:

- Add a transport thinly layered over AutomationCommand and AutomationSnapshot
- Ship the first external adapter as a dedicated stdio binary behind a feature gate
- Document architecture command model snapshot model and non goals in repo docs

**Acceptance Criteria**:

- cargo run -p game_app remains unchanged
- The agent transport activates only through the dedicated feature or binary
- Representative commands and snapshots round trip through stdio JSON Lines

**Tests**:

- integration: stdio transport round trip for representative commands
- integration: transport error responses stay structured and deterministic

#### Code Intent

- **CI-M-003-001** `crates/game_app/Cargo.toml::feature wiring`: Add feature gated transport and binary wiring so agent control remains opt in and cargo run -p game_app stays unchanged (refs: DL-005)
- **CI-M-003-002** `crates/game_app/src/automation_transport.rs::stdio JSONL adapter`: Serialize AutomationCommand and AutomationSnapshot over a thin transport layer that owns framing and error envelopes but not gameplay rules (refs: DL-002, DL-005)
- **CI-M-003-003** `crates/game_app/src/bin/game_app_agent.rs::agent binary main`: Provide a dedicated agent entry point that boots the automation surface reads stdio requests writes structured snapshot responses and leaves the GUI binary path untouched (refs: DL-001, DL-005)
- **CI-M-003-004** `crates/game_app/tests/automation_transport.rs::transport contract coverage`: Verify representative stdio command and snapshot round trips plus structured failure responses for invalid commands or unavailable saves (refs: DL-005, DL-006)
- **CI-M-003-005** `plans/agent-testing.md::plan documentation`: Document the phased automation architecture milestones invariants risks and non goals in the repository plan style (refs: DL-001, DL-005)
- **CI-M-003-006** `README.md::workspace automation notes`: Describe the opt in automation seam and clarify that external agent transport remains optional rather than part of the default player workflow (refs: DL-004, DL-005)
- **CI-M-003-007** `crates/game_app/README.md::crate automation guide`: Describe harness construction snapshot invariants semantic command routing and the feature gated agent binary for maintainers (refs: DL-002, DL-004, DL-005)

#### Code Changes

**CC-M-003-001** (crates/game_app/Cargo.toml) - implements CI-M-003-001

**Code:**

```diff
--- a/crates/game_app/Cargo.toml
+++ b/crates/game_app/Cargo.toml
@@ -9,10 +9,20 @@ path = "src/lib.rs"
 [[bin]]
 name = "game_app"
 path = "src/main.rs"

+[[bin]]
+name = "game_app_agent"
+path = "src/bin/game_app_agent.rs"
+required-features = ["automation-transport"]
+
+[features]
+default = []
+automation-transport = ["dep:serde", "dep:serde_json"]
+
 [dependencies]
 bevy.workspace = true
 chess_core = { path = "../chess_core" }
 chess_persistence = { path = "../chess_persistence" }
 engine_uci = { path = "../engine_uci" }
+serde = { workspace = true, optional = true }
+serde_json = { workspace = true, optional = true }

 [dev-dependencies]
 tempfile = "3.15.0"
```

**Documentation:**

```diff
--- a/crates/game_app/Cargo.toml
+++ b/crates/game_app/Cargo.toml
@@ -1,3 +1,6 @@
+# Feature-gated transport keeps the agent binary opt-in and preserves the GUI
+# startup contract for the default `game_app` target. (ref: DL-005)
+
 [package]
 name = "game_app"
 version.workspace = true

```


**CC-M-003-002** (crates/game_app/src/automation_transport.rs) - implements CI-M-003-002

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/automation_transport.rs
@@ -0,0 +1,66 @@
+use std::io::{self, BufRead, Write};
+
+use crate::{AutomationCommand, AutomationHarness, AutomationSnapshot};
+
+use serde::{Deserialize, Serialize};
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct AutomationRequest {
+    pub command: AutomationCommand,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct AutomationResponse {
+    pub snapshot: Option<AutomationSnapshot>,
+    pub error: Option<AutomationTransportError>,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct AutomationTransportError {
+    pub code: AutomationTransportErrorCode,
+    pub message: String,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum AutomationTransportErrorCode {
+    InvalidRequest,
+    CommandRejected,
+}
+
+impl AutomationResponse {
+    fn success(snapshot: AutomationSnapshot) -> Self {
+        Self {
+            snapshot: Some(snapshot),
+            error: None,
+        }
+    }
+
+    fn invalid_request(message: String) -> Self {
+        Self {
+            snapshot: None,
+            error: Some(AutomationTransportError {
+                code: AutomationTransportErrorCode::InvalidRequest,
+                message,
+            }),
+        }
+    }
+
+    fn command_rejected(message: String) -> Self {
+        Self {
+            snapshot: None,
+            error: Some(AutomationTransportError {
+                code: AutomationTransportErrorCode::CommandRejected,
+                message,
+            }),
+        }
+    }
+}
+
+pub fn run_stdio_session<R: BufRead, W: Write>(
+    reader: R,
+    mut writer: W,
+    harness: &mut AutomationHarness,
+) -> io::Result<()> {
+    for line in reader.lines() {
+        let line = line?;
+        if line.trim().is_empty() {
+            continue;
+        }
+
+        let response = match serde_json::from_str::<AutomationRequest>(&line) {
+            Ok(request) => match harness.try_submit(request.command) {
+                Ok(snapshot) => AutomationResponse::success(snapshot),
+                Err(error) => AutomationResponse::command_rejected(error.to_string()),
+            },
+            Err(error) => AutomationResponse::invalid_request(error.to_string()),
+        };
+
+        serde_json::to_writer(&mut writer, &response).map_err(io::Error::other)?;
+        writer.write_all(b"\n")?;
+    }
+    writer.flush()
+}
```

**Documentation:**

```diff
--- a/crates/game_app/src/automation_transport.rs
+++ b/crates/game_app/src/automation_transport.rs
@@ -1,3 +1,6 @@
+// JSON Lines transport frames the shared automation contract while gameplay
+// semantics stay in the in-process harness. (refs: DL-002, DL-005)
+
 use std::io::{self, BufRead, Write};
 
 use crate::{AutomationCommand, AutomationHarness, AutomationSnapshot};

```


**CC-M-003-003** (crates/game_app/src/bin/game_app_agent.rs) - implements CI-M-003-003

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/bin/game_app_agent.rs
@@ -0,0 +1,13 @@
+// Dedicated agent entry point for the feature-gated automation transport.
+// The GUI binary remains the player-facing startup path. (ref: DL-005)
+
+use std::io::{self, BufReader};
+
+fn main() -> io::Result<()> {
+    let mut harness = game_app::AutomationHarness::new(None).with_semantic_automation();
+    let stdin = io::stdin();
+    let stdout = io::stdout();
+
+    harness.boot_to_main_menu();
+    game_app::automation_transport::run_stdio_session(
+        BufReader::new(stdin.lock()),
+        stdout.lock(),
+        &mut harness,
+    )
+}
```

**Documentation:**

```diff


```


**CC-M-003-004** (crates/game_app/tests/automation_transport.rs) - implements CI-M-003-004

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/automation_transport.rs
@@ -0,0 +1,79 @@
+#![cfg(feature = "automation-transport")]
+
+use std::io::Cursor;
+
+use tempfile::tempdir;
+
+use game_app::{
+    automation_transport::{
+        AutomationRequest, AutomationResponse, AutomationTransportErrorCode, run_stdio_session,
+    },
+    AutomationCommand, AutomationHarness, AutomationNavigationAction, AutomationSaveAction,
+    AutomationScreen,
+};
+
+fn decode(output: Vec<u8>) -> Vec<AutomationResponse> {
+    String::from_utf8(output)
+        .expect("transport output should stay utf8")
+        .lines()
+        .map(|line| {
+            serde_json::from_str::<AutomationResponse>(line)
+                .expect("each response line should parse")
+        })
+        .collect()
+}
+
+#[test]
+fn stdio_roundtrips_snapshots_for_representative_commands() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut harness =
+        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
+    harness.boot_to_main_menu();
+
+    let requests = [
+        AutomationRequest {
+            command: AutomationCommand::Navigation(AutomationNavigationAction::OpenSetup),
+        },
+        AutomationRequest {
+            command: AutomationCommand::Navigation(AutomationNavigationAction::StartNewMatch),
+        },
+        AutomationRequest {
+            command: AutomationCommand::Step { frames: 3 },
+        },
+    ];
+    let mut input = String::new();
+    for request in requests {
+        input.push_str(&serde_json::to_string(&request).expect("request should serialize"));
+        input.push('\n');
+    }
+
+    let mut output = Vec::new();
+    run_stdio_session(Cursor::new(input.into_bytes()), &mut output, &mut harness)
+        .expect("stdio transport should succeed");
+
+    let responses = decode(output);
+    assert_eq!(responses.len(), 3);
+    assert!(responses.iter().all(|response| response.error.is_none()));
+    assert_eq!(
+        responses.last().and_then(|response| response.snapshot.as_ref()).map(|snapshot| snapshot.screen),
+        Some(AutomationScreen::InMatch)
+    );
+}
+
+#[test]
+fn stdio_returns_structured_errors_for_invalid_json_and_missing_save_selection() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut harness =
+        AutomationHarness::new(Some(root.path().to_path_buf())).with_semantic_automation();
+    harness.boot_to_main_menu();
+
+    let valid = serde_json::to_string(&AutomationRequest {
+        command: AutomationCommand::Save(AutomationSaveAction::LoadSelected),
+    })
+    .expect("request should serialize");
+    let input = format!("{{not json}}\n{valid}\n");
+
+    let mut output = Vec::new();
+    run_stdio_session(Cursor::new(input.into_bytes()), &mut output, &mut harness)
+        .expect("transport should still emit structured errors");
+
+    let responses = decode(output);
+    assert_eq!(responses[0].error.as_ref().map(|error| error.code), Some(AutomationTransportErrorCode::InvalidRequest));
+    assert_eq!(responses[1].error.as_ref().map(|error| error.code), Some(AutomationTransportErrorCode::CommandRejected));
+}
```

**Documentation:**

```diff
--- a/crates/game_app/tests/automation_transport.rs
+++ b/crates/game_app/tests/automation_transport.rs
@@ -1,3 +1,6 @@
+// Transport contract coverage locks the request and response envelope without
+// depending on a native window. (refs: DL-005, DL-006)
+
 #![cfg(feature = "automation-transport")]

 use std::io::Cursor;

```


**CC-M-003-005** (plans/agent-testing.md) - implements CI-M-003-005

**Code:**

```diff
--- a/plans/agent-testing.md
            +++ b/plans/agent-testing.md
            @@ -10,6 +10,27 @@

             [Diagram pending Technical Writer rendering: DIAG-001]

+```text
+AutomationCommand -> AutomationHarness -> AutomationPlugin
+                   -> MenuAction / SaveLoadRequest / MatchSession
+                   -> AutomationSnapshot
+```
+
+### Command Model
+
+- `AutomationCommand` stays at the semantic shell and match level: navigation, save/load, settings, match actions, and confirmations.
+- `AutomationHarness::try_submit` executes one command and returns a fresh `AutomationSnapshot`.
+- `AutomationPlugin` owns the in-process queue so integration tests and the optional `stdio` adapter stay on one dispatch path.
+
+### Snapshot Model
+
+- `AutomationSnapshot` exposes screen state, modal shell state, player-visible match state, and persisted save metadata.
+- Snapshot fields intentionally use FEN, legal targets, pending promotion, last move, and save summaries instead of ECS identifiers or render details.
+- Recovery availability and shell status stay in the snapshot so external adapters do not need direct resource reads.
+
+### Non-Goals
+
+- Native desktop click and key automation as a primary interface.
+- Network-first control surfaces before the in-process contract exists.
+- Separate command models for local harness execution and optional transport adapters.
+
             ## Planning Context

             ### Decision Log
```

**Documentation:**

```diff
--- a/plans/agent-testing.md
+++ b/plans/agent-testing.md
@@ -171,3 +171,6 @@
 - `W-001`: `M-001`
 - `W-002`: `M-002`
 - `W-003`: `M-003`
+## Contract Notes
+
+`AutomationCommand` and `AutomationSnapshot` define the behavior surface. Transport adapters frame that contract over a chosen medium, while gameplay semantics remain inside `game_app`. (refs: DL-001, DL-005)

```


**CC-M-003-006** (README.md) - implements CI-M-003-006

**Code:**

```diff
--- a/README.md
            +++ b/README.md
            @@ -14,6 +14,7 @@
             ## Planning Docs

             - [Milestones](/home/franky/repos/3d-chess/plans/milestones.md)
             - [Implementation Plan](/home/franky/repos/3d-chess/plans/implementation-plan.md)
            +- [Agent Testing Plan](/home/franky/repos/3d-chess/plans/agent-testing.md)

             ## Workspace Layout

            @@ -33,6 +34,7 @@
             - `cargo clippy --workspace --all-targets -- -D warnings`
             - `cargo test --workspace`
             - `cargo run -p game_app`
            +- `cargo run -p game_app --features automation-transport --bin game_app_agent`
             - `cargo build --workspace --release`

             ## Architecture Boundaries
            @@ -42,6 +44,7 @@
             - `chess_core` stays pure Rust and remains the only gameplay authority for rules, legality, and outcomes.
             - `chess_persistence` owns versioned snapshot formats, file-backed repository I/O, platform app-data roots, and the narrow M3 settings contract.
             - `game_app` keeps top-level routing coarse and renders menus, pause overlays, promotion UI, save/load flow, and result screens around `MatchSession`.
+            - Agent automation remains opt-in: `AutomationHarness` and the feature-gated `game_app_agent` binary route semantic commands and snapshots through `game_app` without changing `cargo run -p game_app`.
             - `engine_uci` reserves the Stockfish/UCI integration boundary instead of leaking AI concerns into the shipped local shell.
```

**Documentation:**

```diff
--- a/README.md
+++ b/README.md
@@ -66,3 +66,6 @@
 - M1: `chess_core` legal move generation, move application, check and mate resolution, draw semantics, castling, en passant, promotion, and FEN support.
 - M2: `game_app` local match startup, board and piece synchronization, square picking, promotion flow, claim-draw flow, and result transitions.
 - M3: `chess_persistence` file-backed saves, recovery state, and shell settings; `game_app` main-menu setup, pause overlays, save/load UX, recovery resume, and result rematch flow; CI packaging and packaged startup smoke on both desktop targets.
+## Agent Automation
+
+`game_app` supports an opt-in headless automation seam for tests and local agents. A feature-gated `game_app_agent` binary exposes the same semantic contract over `stdio` without changing the default GUI workflow. (refs: DL-004, DL-005)

```


**CC-M-003-007** (crates/game_app/README.md) - implements CI-M-003-007

**Code:**

```diff
--- a/crates/game_app/README.md
            +++ b/crates/game_app/README.md
            @@ -13,6 +13,12 @@
             - `AppShellPlugin` renders the main menu, pause surfaces, promotion overlay, and result flow from modal shell resources while board and piece rendering stay in dedicated scene plugins.
             - Board rendering, piece placement, and cursor picking all share the same coordinate helpers in `board_coords.rs`.

+## Automation Seam
+
+- `build_headless_app` composes the shipped shell graph without a native window for integration tests and agent harnesses.
+- `AutomationHarness` boots that graph and captures semantic snapshots of screen, shell, save/load, and match state.
+- `AutomationPlugin` owns command dispatch so automation, shell buttons, and raw input stay on one semantic path.
+- The `game_app_agent` binary adds JSON Lines `stdio` transport only when the `automation-transport` feature is enabled.
+
             ## Invariants

             - `chess_core::GameState` remains the source of truth for legal moves, side to move, checks, and terminal outcomes.
```

**Documentation:**

```diff
--- a/crates/game_app/README.md
+++ b/crates/game_app/README.md
@@ -31,3 +31,6 @@
 - The board and pieces stay procedural so effort goes into interaction correctness, persistence flow, and test coverage instead of asset pipelines.
 - Picking uses internal camera-ray math because the board plane is fixed and deterministic; a generic picking dependency would add surface area without helping the current use case.
 - Claimable draws are resolved in `MatchSession` so the shell can end claimable games without widening `chess_core` beyond the persisted session contract it already owns.
+## Automation Contract
+
+`AutomationSnapshot` reads `MatchSession`, `ShellMenuState`, `SaveLoadState`, and `RecoveryBannerState`, so observers stay aligned with player-visible state instead of ECS internals. `AutomationMatchAction`, `AutomationNavigationAction`, and `AutomationSaveAction` route through the same helpers that shell buttons and raw input use. (refs: DL-002, DL-003)

```


## Execution Waves

- W-001: M-001
- W-002: M-002
- W-003: M-003
## Contract Notes

`AutomationCommand` and `AutomationSnapshot` define the behavior surface. Transport adapters frame that contract over a chosen medium, while gameplay semantics remain inside `game_app`. (refs: DL-001, DL-005)
