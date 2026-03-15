# M3 Playable Game Loop Plan

## Overview

3d-chess has a playable M2 local match loop but still lacks the product-shell persistence recovery and CI artifact behavior required for M3.

**Approach**: Extend chess_persistence into a file-backed session store, route game_app through explicit new/load/resume shell intents with modal menus, and package portable Windows/Linux artifacts with boot-smoke validation in CI.

### M3 Runtime Overview

[Diagram pending Technical Writer rendering: DIAG-001]

## Planning Context

### Decision Log

| ID | Decision | Reasoning Chain |
|---|---|---|
| DL-001 | Keep coarse Bevy screen states and route M3 shell flow through explicit launch intents plus modal shell resources. | M3 only needs Boot/MainMenu/MatchLoading/InMatch/MatchResult as top-level screen states -> local setup pause save-load and settings fit as modal shell resources over those coarse routes -> launch intents keep MatchLoading explicit without adding one app state per popup. |
| DL-002 | Extend chess_persistence into a file-backed saved-session repository that stores versioned domain state and shell metadata together. | M3 must ship UI save/load and interrupted-session recovery -> GameState alone omits pending-promotion and save-list metadata -> a repository API keeps filesystem and compatibility logic out of Bevy systems. |
| DL-003 | Keep interrupted-session recovery separate from manual saves through a dedicated autosave record and resume policy. | Crash or quit recovery must resume the last interrupted match with minimal friction -> manual saves need stable user-controlled history -> separate records prevent autosave churn from overwriting deliberate saves. |
| DL-004 | Restore matches from persisted session snapshots rather than serializing ECS world state. | chess_core already owns legal state and outcome evaluation -> Bevy entities and UI are projections that can be rebuilt -> domain-first snapshots preserve deterministic restore across platforms and future UI changes. |
| DL-005 | Ship exactly three persisted M3 settings: startup recovery behavior, destructive-action confirmations, and display mode; defer broader options beyond this milestone. | M3 must close the local product shell without absorbing M5 polish scope -> the shell only needs launch recovery policy, confirmations for abandon delete and overwrite actions, and a basic windowed-versus-fullscreen display choice -> audio, graphics, controls, accessibility, and other presentation options stay deferred beyond M3. |
| DL-006 | Package portable Windows and Linux artifacts in CI and prove boot with scripted startup smoke checks instead of installer or signing work. | M3 acceptance is CI-produced bootable artifacts -> portable archives meet that bar on GitHub-hosted runners -> installer signing and release-channel work stay deferred to M6. |
| DL-007 | Keep repository I/O and recovery policy in dedicated game_app shell plugins instead of AppShellPlugin or MatchSession. | AppShellPlugin already owns camera and shared shell UI -> MatchSession is the Bevy-to-domain gameplay bridge -> dedicated menu and save-load plugins isolate persistence concerns and keep shell growth testable. |
| DL-008 | Default persistence storage to standard per-platform app-data directories, while keeping injected roots for tests and tooling overrides. | Saved sessions and recovery state need a predictable user-visible home on Windows and Linux -> standard app-data locations avoid ad hoc working-directory writes and match desktop expectations -> injected roots preserve deterministic tests and packaging-smoke flexibility without changing the runtime default. |

### Rejected Alternatives

| Alternative | Why Rejected |
|---|---|
| Persist raw ECS world state for save and load. | Derived visuals and entity identifiers would drift from chess_core authority while domain-first snapshots remain deterministic and portable. (ref: DL-004) |
| Add a top-level AppScreenState for every popup such as save load settings and recovery. | Popup-specific states would turn modal shell UX into routing sprawl and make later shell growth harder to reason about. (ref: DL-001) |
| Let MatchSession read and write save files directly. | Filesystem policy and slot management would leak into the Bevy-to-domain bridge and make gameplay behavior depend on storage concerns. (ref: DL-007) |
| Store interrupted-session recovery in the same slot set as manual saves. | Automatic crash or quit recovery and user-managed saves have different overwrite and presentation rules. (ref: DL-003) |
| Add installers signing and release-channel work during M3. | Portable artifacts and startup smoke checks satisfy the M3 acceptance bar while full distribution engineering is reserved for M6. (ref: DL-006) |

### Constraints

- [doc-derived] MUST: keep chess_core authoritative for rules and let Bevy mirror domain state only (plans/milestones.md; plans/implementation-plan.md; crates/game_app/README.md)
- [doc-derived] MUST: save versioned domain and session snapshots rather than ECS world state (plans/milestones.md; plans/implementation-plan.md)
- [task-derived] MUST: keep M3 scope inside crates/game_app shell flow crates/chess_persistence snapshot and storage APIs game_app tests and .github/workflows/ci.yml artifact work
- [doc-derived] MUST: cover menu flow match setup pause and result handling shipped save-load UI interrupted-session recovery and Windows/Linux packaged builds in M3 (plans/milestones.md#M3; plans/implementation-plan.md#M3)
- [doc-derived] MUST-NOT: pull Stockfish or broader AI lifecycle work into M3 because that remains M4 scope (plans/milestones.md; plans/implementation-plan.md)
- [doc-derived] MUST-NOT: pull installer signing or full release-process work into M3 because portable bootable artifacts are sufficient before M6 (plans/milestones.md; task context)
- [inferred] SHOULD: keep top-level app state coarse and use modal or orthogonal shell resources for transient menus settings and save dialogs so shell growth does not turn into routing sprawl (app.rs state enum; task context)
- [doc-derived] MUST: stay desktop-native Rust plus Bevy for Windows and Linux with no JS TS or web delivery (plans/milestones.md)

### Known Risks

- **AppShellPlugin may become a shell and persistence god-object as pause save load and recovery UI arrive.**: Move repository access and shell-policy resources into dedicated menu and save-load plugins so AppShellPlugin stays focused on presentation and shared state transitions.
- **Interrupted-session recovery may lose legality-critical interaction state such as pending promotion or resume the wrong match.**: Extend the persisted session contract to include pending-promotion metadata save kind and timestamps and verify restore flows against real files.
- **MatchLoading may still reset to a fresh starting position when the user intended load or resume.**: Route every new load and resume path through an explicit launch-intent resource consumed by MatchLoading and cover each branch in integration tests.
- **Bevy startup smoke checks may be flaky on headless GitHub runners.**: Use OS-specific launch scripts with bounded timeouts explicit process cleanup and packaged-artifact paths instead of compile-only validation.
- **Platform path or permission differences may make save-load behavior diverge between Windows and Linux.**: Centralize standard app-data path resolution in chess_persistence and exercise repository tests through injected temporary roots plus app-level recovery coverage.

## Invisible Knowledge

### System

M2 already delivered a real local playable loop, so M3 is not about more chess rules; it is about turning that loop into a product shell that can launch pause persist recover and package cleanly without violating chess_core authority.

### Invariants

- MatchSession remains the only Bevy-facing bridge to chess_core and storage code never becomes a second gameplay authority.
- Save-load and recovery restore the same legal chess position plus legality-critical shell state across Windows and Linux.
- Match launch always flows through an explicit new load or resume intent and MatchLoading never guesses.
- Interrupted-session recovery and manual saves remain separate user concepts with separate overwrite rules.
- M3 CI acceptance requires packaged binaries that boot successfully and not just successful release compilation.

### Tradeoffs

- Prefer coarse app states plus modal resources over adding a top-level screen for every popup.
- Prefer a small explicit settings contract over a broad options surface that belongs to later polish milestones.
- Prefer file-backed repository APIs with injected roots over pushing platform path logic into UI systems.
- Prefer portable archives and scripted startup smoke over installer and signing complexity during M3.

## Milestones

### Milestone 1: Saved Session Contract and Repository

**Files**: Cargo.lock, crates/chess_persistence/Cargo.toml, crates/chess_persistence/src/lib.rs, crates/chess_persistence/src/snapshot.rs, crates/chess_persistence/src/store.rs, crates/chess_persistence/tests/session_store.rs

**Flags**: persistence, file-io, recovery

**Requirements**:

- Persist versioned saved-session metadata needed for load and recovery
- Add file-backed repository APIs for manual saves recovery records and settings with injectable roots
- Keep restore boundaries domain-first and independent from Bevy ECS state

**Acceptance Criteria**:

- Real-file roundtrips preserve game state and legality-critical shell metadata
- Repository APIs create list load delete manual saves and independently manage recovery state
- Persistence tests use injected temp directories instead of project-relative paths

**Tests**:

- integration: manual save list load delete roundtrip
- integration: recovery record and settings roundtrip

#### Code Intent

- **CI-M-001-001** `crates/chess_persistence/src/snapshot.rs::saved-session schema`: Extend the versioned persistence schema to encode active-match state plus shell metadata such as pending promotion save kind timestamps and recovery identity without leaking Bevy-specific entity state. (refs: DL-002, DL-003, DL-004)
- **CI-M-001-002** `crates/chess_persistence/src/store.rs::SessionStore`: Provide file-backed repository methods for saving listing loading deleting manual saves and for storing loading clearing recovery snapshots and shell settings through one injected root boundary, defaulting runtime storage to standard platform app-data directories. (refs: DL-002, DL-003, DL-005, DL-008)
- **CI-M-001-003** `crates/chess_persistence/src/lib.rs::public exports and tests`: Re-export the saved-session repository surface and add behavior tests that prove persisted sessions restore the same playable domain state from real files. (refs: DL-002, DL-004)
- **CI-M-001-004** `crates/chess_persistence/Cargo.toml::dependencies`: Add only the persistence dependencies needed for standard platform app-data path resolution, timestamps, and atomic file writes inside chess_persistence. (refs: DL-002, DL-005, DL-008)
- **CI-M-001-005** `crates/chess_persistence/tests/session_store.rs::filesystem repository coverage`: Exercise manual saves recovery records and settings against injected temporary directories so the repository contract verifies both real filesystem behavior and the default platform-path policy. (refs: DL-002, DL-003, DL-005, DL-008)
- **CI-M-001-006** `Cargo.lock::workspace lockfile`: Refresh Cargo.lock for the chess_persistence filesystem and timestamp dependencies introduced in M1 so the tracked workspace lockfile is already aligned before later milestones extend game_app. (refs: DL-002, DL-007, DL-008)

#### Code Changes

**CC-M-001-001** (crates/chess_persistence/src/snapshot.rs) - implements CI-M-001-001

**Code:**

```diff
--- a/crates/chess_persistence/src/snapshot.rs
+++ b/crates/chess_persistence/src/snapshot.rs
@@ -1,17 +1,54 @@
-use chess_core::GameState;
+use chess_core::{GameState, Move, Square};
 use serde::{Deserialize, Serialize};
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
 pub enum SaveFormatVersion {
+    V1,
     #[default]
-    V1,
+    V2,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
+pub enum SaveKind {
+    #[default]
+    Manual,
+    Recovery,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+pub enum ClaimedDrawSnapshot {
+    ThreefoldRepetition,
+    FiftyMoveRule,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+pub struct PendingPromotionSnapshot {
+    pub from: Square,
+    pub to: Square,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
+pub struct SnapshotShellState {
+    pub selected_square: Option<Square>,
+    pub pending_promotion: Option<PendingPromotionSnapshot>,
+    pub last_move: Option<Move>,
+    pub claimed_draw: Option<ClaimedDrawSnapshot>,
+    pub dirty_recovery: bool,
 }
 
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
 pub struct SnapshotMetadata {
     pub label: String,
     pub created_at_utc: Option<String>,
+    #[serde(default)]
+    pub updated_at_utc: Option<String>,
     pub notes: Option<String>,
+    #[serde(default)]
+    pub save_kind: SaveKind,
+    #[serde(default)]
+    pub session_id: String,
+    #[serde(default)]
+    pub recovery_key: Option<String>,
 }
 
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
@@ -19,15 +56,28 @@
     pub version: SaveFormatVersion,
     pub game_state: GameState,
     pub metadata: SnapshotMetadata,
+    #[serde(default)]
+    pub shell_state: SnapshotShellState,
 }
 
 impl GameSnapshot {
     #[must_use]
     pub fn new(game_state: GameState, metadata: SnapshotMetadata) -> Self {
+        Self::from_parts(game_state, metadata, SnapshotShellState::default())
+    }
+
+    #[must_use]
+    pub fn from_parts(
+        game_state: GameState,
+        metadata: SnapshotMetadata,
+        shell_state: SnapshotShellState,
+    ) -> Self {
         Self {
-            version: SaveFormatVersion::V1,
+            version: SaveFormatVersion::V2,
             game_state,
             metadata,
+            // Persist only domain and interaction state so recovery survives future UI rewrites.
+            shell_state,
         }
     }
 
@@ -35,4 +85,14 @@
     pub fn restore_game_state(&self) -> GameState {
         self.game_state.clone()
     }
+
+    #[must_use]
+    pub fn metadata(&self) -> &SnapshotMetadata {
+        &self.metadata
+    }
+
+    #[must_use]
+    pub fn shell_state(&self) -> &SnapshotShellState {
+        &self.shell_state
+    }
 }

```

**Documentation:**

```diff
--- a/crates/chess_persistence/src/snapshot.rs
+++ b/crates/chess_persistence/src/snapshot.rs
@@ -1,5 +1,8 @@
+//! Versioned saved-session snapshots for manual saves and interrupted-session recovery.
+//! Domain state stays authoritative while shell metadata captures legality-critical interaction state. (ref: DL-002) (ref: DL-004)
+
 use chess_core::{GameState, Move, Square};
 use serde::{Deserialize, Serialize};
 
@@ -31,6 +34,10 @@ pub struct PendingPromotionSnapshot {
     pub to: Square,
 }
 
+/// Captures the minimal interaction state that must survive restore without serializing Bevy entities.
+/// Pending promotion and recovery dirtiness live here because they affect legal resume behavior. (ref: DL-002) (ref: DL-004)
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
 pub struct SnapshotShellState {
     pub selected_square: Option<Square>,
@@ -66,6 +73,10 @@ impl GameSnapshot {
         Self::from_parts(game_state, metadata, SnapshotShellState::default())
     }
 
+    /// Builds a snapshot from domain state plus shell metadata so UI projections can be rebuilt after load.
+    /// Restore behavior never depends on serializing Bevy world state. (ref: DL-004)
     #[must_use]
     pub fn from_parts(
         game_state: GameState,

```


**CC-M-001-002** (crates/chess_persistence/src/store.rs) - implements CI-M-001-002

**Code:**

```diff
--- /dev/null
+++ b/crates/chess_persistence/src/store.rs
@@ -0,0 +1,315 @@
+use std::fs;
+use std::io::{self, Write};
+use std::path::{Path, PathBuf};
+
+use directories::ProjectDirs;
+use serde::de::DeserializeOwned;
+use serde::{Deserialize, Serialize};
+use tempfile::NamedTempFile;
+use time::OffsetDateTime;
+use time::format_description::well_known::Rfc3339;
+
+use crate::{GameSnapshot, SaveKind};
+
+const APP_DATA_QUALIFIER: &str = "io";
+const APP_DATA_ORG: &str = "franky";
+const APP_DATA_NAME: &str = "3d-chess";
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
+pub enum RecoveryStartupPolicy {
+    Resume,
+    Ignore,
+    #[default]
+    Ask,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
+pub enum DisplayMode {
+    #[default]
+    Windowed,
+    Fullscreen,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct ConfirmActionSettings {
+    pub overwrite_save: bool,
+    pub delete_save: bool,
+    pub abandon_match: bool,
+}
+
+impl Default for ConfirmActionSettings {
+    fn default() -> Self {
+        Self {
+            overwrite_save: true,
+            delete_save: true,
+            abandon_match: true,
+        }
+    }
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
+pub struct ShellSettings {
+    pub recovery_policy: RecoveryStartupPolicy,
+    pub confirm_actions: ConfirmActionSettings,
+    pub display_mode: DisplayMode,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct SavedSessionSummary {
+    pub slot_id: String,
+    pub label: String,
+    pub created_at_utc: Option<String>,
+    pub save_kind: SaveKind,
+}
+
+impl SavedSessionSummary {
+    #[must_use]
+    pub fn from_snapshot(snapshot: &GameSnapshot) -> Self {
+        Self {
+            slot_id: snapshot.metadata.session_id.clone(),
+            label: snapshot.metadata.label.clone(),
+            created_at_utc: snapshot.metadata.created_at_utc.clone(),
+            save_kind: snapshot.metadata.save_kind,
+        }
+    }
+}
+
+#[derive(Debug)]
+pub enum StoreError {
+    Io(io::Error),
+    Serialization(serde_json::Error),
+    MissingPlatformDir,
+}
+
+impl From<io::Error> for StoreError {
+    fn from(error: io::Error) -> Self {
+        Self::Io(error)
+    }
+}
+
+impl From<serde_json::Error> for StoreError {
+    fn from(error: serde_json::Error) -> Self {
+        Self::Serialization(error)
+    }
+}
+
+pub type StoreResult<T> = Result<T, StoreError>;
+
+#[derive(Debug, Clone)]
+pub struct SessionStore {
+    root: PathBuf,
+}
+
+impl SessionStore {
+    #[must_use]
+    pub fn new(root: impl Into<PathBuf>) -> Self {
+        Self { root: root.into() }
+    }
+
+    pub fn runtime() -> StoreResult<Self> {
+        Ok(Self::new(Self::default_root()?))
+    }
+
+    pub fn default_root() -> StoreResult<PathBuf> {
+        let Some(project_dirs) = ProjectDirs::from(APP_DATA_QUALIFIER, APP_DATA_ORG, APP_DATA_NAME)
+        else {
+            return Err(StoreError::MissingPlatformDir);
+        };
+
+        Ok(project_dirs.data_dir().to_path_buf())
+    }
+
+    #[must_use]
+    pub fn root(&self) -> &Path {
+        &self.root
+    }
+
+    pub fn list_manual_saves(&self) -> StoreResult<Vec<SavedSessionSummary>> {
+        self.ensure_layout()?;
+        let mut saves = Vec::new();
+        for entry in fs::read_dir(self.manual_saves_dir())? {
+            let entry = entry?;
+            if entry.file_type()?.is_file() {
+                let snapshot: GameSnapshot = self.read_json(&entry.path())?;
+                saves.push(SavedSessionSummary::from_snapshot(&snapshot));
+            }
+        }
+
+        saves.sort_by(|left, right| right.created_at_utc.cmp(&left.created_at_utc));
+        Ok(saves)
+    }
+
+    pub fn save_manual(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
+        self.ensure_layout()?;
+        let now = now_utc();
+        if snapshot.metadata.label.trim().is_empty() {
+            snapshot.metadata.label = format!("Manual Save {now}");
+        }
+        let requested_slot = snapshot.metadata.session_id.trim();
+        if requested_slot.is_empty() {
+            // Fresh manual saves allocate unique history slots; explicit slot ids are overwrite targets.
+            snapshot.metadata.session_id =
+                self.next_manual_slot_id(&snapshot.metadata.label)?;
+        }
+        snapshot.metadata.save_kind = SaveKind::Manual;
+        if snapshot.metadata.created_at_utc.is_none() {
+            snapshot.metadata.created_at_utc = Some(now.clone());
+        }
+        snapshot.metadata.updated_at_utc = Some(now);
+
+        let summary = SavedSessionSummary::from_snapshot(&snapshot);
+        self.write_json_atomic(&self.manual_save_path(&summary.slot_id), &snapshot)?;
+        Ok(summary)
+    }
+
+    pub fn load_manual(&self, slot_id: &str) -> StoreResult<GameSnapshot> {
+        self.read_json(&self.manual_save_path(slot_id))
+    }
+
+    pub fn delete_manual(&self, slot_id: &str) -> StoreResult<()> {
+        let path = self.manual_save_path(slot_id);
+        if path.exists() {
+            fs::remove_file(path)?;
+        }
+        Ok(())
+    }
+
+    pub fn store_recovery(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
+        self.ensure_layout()?;
+        let now = now_utc();
+        snapshot.metadata.save_kind = SaveKind::Recovery;
+        snapshot.metadata.session_id = String::from("recovery");
+        snapshot
+            .metadata
+            .recovery_key
+            .get_or_insert_with(|| String::from("autosave"));
+        if snapshot.metadata.created_at_utc.is_none() {
+            snapshot.metadata.created_at_utc = Some(now.clone());
+        }
+        snapshot.metadata.updated_at_utc = Some(now);
+
+        let summary = SavedSessionSummary::from_snapshot(&snapshot);
+        self.write_json_atomic(&self.recovery_file(), &snapshot)?;
+        Ok(summary)
+    }
+
+    pub fn load_recovery(&self) -> StoreResult<Option<GameSnapshot>> {
+        let path = self.recovery_file();
+        if !path.exists() {
+            return Ok(None);
+        }
+
+        self.read_json(&path).map(Some)
+    }
+
+    pub fn clear_recovery(&self) -> StoreResult<()> {
+        let path = self.recovery_file();
+        if path.exists() {
+            fs::remove_file(path)?;
+        }
+        Ok(())
+    }
+
+    pub fn load_settings(&self) -> StoreResult<ShellSettings> {
+        let path = self.settings_file();
+        if !path.exists() {
+            return Ok(ShellSettings::default());
+        }
+
+        self.read_json(&path)
+    }
+
+    pub fn save_settings(&self, settings: &ShellSettings) -> StoreResult<()> {
+        self.ensure_layout()?;
+        self.write_json_atomic(&self.settings_file(), settings)
+    }
+
+    fn ensure_layout(&self) -> StoreResult<()> {
+        fs::create_dir_all(self.manual_saves_dir())?;
+        fs::create_dir_all(self.recovery_dir())?;
+        Ok(())
+    }
+
+    fn next_manual_slot_id(&self, label: &str) -> StoreResult<String> {
+        let base = slugify_label(label);
+        let base = if base.is_empty() {
+            String::from("manual-save")
+        } else {
+            base
+        };
+        let mut candidate = base.clone();
+        let mut suffix = 2;
+
+        while self.manual_save_path(&candidate).exists() {
+            candidate = format!("{base}-{suffix}");
+            suffix += 1;
+        }
+
+        Ok(candidate)
+    }
+
+    fn manual_saves_dir(&self) -> PathBuf {
+        self.root.join("saves")
+    }
+
+    fn manual_save_path(&self, slot_id: &str) -> PathBuf {
+        self.manual_saves_dir().join(format!("{slot_id}.json"))
+    }
+
+    fn recovery_dir(&self) -> PathBuf {
+        self.root.join("recovery")
+    }
+
+    fn recovery_file(&self) -> PathBuf {
+        self.recovery_dir().join("current.json")
+    }
+
+    fn settings_file(&self) -> PathBuf {
+        self.root.join("settings.json")
+    }
+
+    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> StoreResult<T> {
+        let bytes = fs::read(path)?;
+        Ok(serde_json::from_slice(&bytes)?)
+    }
+
+    fn write_json_atomic<T: Serialize>(&self, path: &Path, value: &T) -> StoreResult<()> {
+        let Some(parent) = path.parent() else {
+            return Err(StoreError::Io(io::Error::new(
+                io::ErrorKind::InvalidInput,
+                "path must have a parent directory",
+            )));
+        };
+
+        fs::create_dir_all(parent)?;
+        let mut temp_file = NamedTempFile::new_in(parent)?;
+        let bytes = serde_json::to_vec_pretty(value)?;
+        temp_file.write_all(&bytes)?;
+        temp_file
+            .persist(path)
+            .map_err(|error| StoreError::Io(error.error))?;
+        Ok(())
+    }
+}
+
+fn now_utc() -> String {
+    OffsetDateTime::now_utc()
+        .format(&Rfc3339)
+        .expect("RFC3339 timestamp formatting should be infallible")
+}
+
+fn slugify_label(label: &str) -> String {
+    let slug: String = label
+        .chars()
+        .map(|character| match character {
+            'a'..='z' | '0'..='9' => character,
+            'A'..='Z' => character.to_ascii_lowercase(),
+            _ => '-',
+        })
+        .collect();
+
+    slug.trim_matches('-').to_string()
+}
```

**Documentation:**

```diff
--- a/crates/chess_persistence/src/store.rs
+++ b/crates/chess_persistence/src/store.rs
@@ -1,5 +1,8 @@
+//! File-backed repository for manual saves, interrupted-session recovery, and the shipped shell settings trio.
+//! The repository owns platform paths and atomic I/O so gameplay code only exchanges snapshots. (ref: DL-002) (ref: DL-005) (ref: DL-008)
+
 use std::fs;
 use std::io::{self, Write};
 use std::path::{Path, PathBuf};
@@ -74,6 +77,9 @@ impl From<serde_json::Error> for StoreError {
 
 pub type StoreResult<T> = Result<T, StoreError>;
 
+/// Stores saved-session files behind a single boundary so shell plugins coordinate persistence without turning `MatchSession` into an I/O owner. (ref: DL-007)
 #[derive(Debug, Clone)]
 pub struct SessionStore {
     root: PathBuf,
@@ -84,10 +90,14 @@ impl SessionStore {
     pub fn new(root: impl Into<PathBuf>) -> Self {
         Self { root: root.into() }
     }
 
+    /// Resolves the runtime root from standard per-platform app-data directories so packaged builds avoid working-directory writes. (ref: DL-008)
     pub fn runtime() -> StoreResult<Self> {
         Ok(Self::new(Self::default_root()?))
     }
@@ -117,6 +127,10 @@ impl SessionStore {
         saves.sort_by(|left, right| right.created_at_utc.cmp(&left.created_at_utc));
         Ok(saves)
     }
 
+    /// Persists manual saves as user-controlled history even when labels collide.
+    /// Slot allocation preserves deliberate saves instead of letting recovery policy rewrite them. (ref: DL-003)
     pub fn save_manual(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
         self.ensure_layout()?;
         let now = now_utc();
@@ -159,6 +173,10 @@ impl SessionStore {
         Ok(())
     }
 
+    /// Stores the single interrupted-session record independently from manual save history so startup recovery stays low-friction and overwrite rules stay distinct. (ref: DL-003)
     pub fn store_recovery(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
         self.ensure_layout()?;
         let now = now_utc();
@@ -193,6 +211,9 @@ impl SessionStore {
         Ok(())
     }
 
+    /// Loads the narrow settings contract that governs startup recovery, destructive confirmations, and display mode. (ref: DL-005)
     pub fn load_settings(&self) -> StoreResult<ShellSettings> {
         let path = self.settings_file();
         if !path.exists() {

```


**CC-M-001-003** (crates/chess_persistence/src/lib.rs) - implements CI-M-001-003

**Code:**

```diff
--- a/crates/chess_persistence/src/lib.rs
+++ b/crates/chess_persistence/src/lib.rs
@@ -1,20 +1,33 @@
 pub mod snapshot;
+pub mod store;
 
-pub use snapshot::{GameSnapshot, SaveFormatVersion, SnapshotMetadata};
+pub use snapshot::{
+    ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SaveFormatVersion,
+    SaveKind, SnapshotMetadata, SnapshotShellState,
+};
+pub use store::{
+    ConfirmActionSettings, DisplayMode, RecoveryStartupPolicy, SavedSessionSummary,
+    SessionStore, ShellSettings, StoreError, StoreResult,
+};
 
 #[cfg(test)]
 mod tests {
     use chess_core::{GameState, GameStatus, Move, Square};
 
-    use crate::{GameSnapshot, SnapshotMetadata};
+    use crate::{
+        ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SaveKind,
+        SnapshotMetadata, SnapshotShellState,
+    };
 
     #[test]
-    fn snapshot_roundtrips_and_preserves_legal_behavior() {
+    fn snapshot_roundtrips_domain_state_and_shell_metadata() {
         let start = GameState::starting_position();
         let e2 = Square::from_algebraic("e2").expect("e2 must be valid");
         let e4 = Square::from_algebraic("e4").expect("e4 must be valid");
         let c7 = Square::from_algebraic("c7").expect("c7 must be valid");
         let c5 = Square::from_algebraic("c5").expect("c5 must be valid");
+        let e7 = Square::from_algebraic("e7").expect("e7 must be valid");
+        let e8 = Square::from_algebraic("e8").expect("e8 must be valid");
 
         let after_e4 = start
             .apply_move(Move::new(e2, e4))
@@ -23,12 +36,23 @@
             .apply_move(Move::new(c7, c5))
             .expect("reply pawn move should be legal");
 
-        let snapshot = GameSnapshot::new(
+        let snapshot = GameSnapshot::from_parts(
             after_c5.clone(),
             SnapshotMetadata {
                 label: String::from("opening"),
                 created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
+                updated_at_utc: Some(String::from("2026-03-15T00:05:00Z")),
                 notes: Some(String::from("Persist opening state")),
+                save_kind: SaveKind::Recovery,
+                session_id: String::from("match-opening"),
+                recovery_key: Some(String::from("autosave")),
+            },
+            SnapshotShellState {
+                selected_square: Some(e7),
+                pending_promotion: Some(PendingPromotionSnapshot { from: e7, to: e8 }),
+                last_move: Some(Move::new(c7, c5)),
+                claimed_draw: Some(ClaimedDrawSnapshot::ThreefoldRepetition),
+                dirty_recovery: true,
             },
         );
 
@@ -42,5 +66,15 @@
         assert_eq!(restored.to_fen(), after_c5.to_fen());
         assert_eq!(restored.legal_moves(), after_c5.legal_moves());
         assert!(matches!(restored.status(), GameStatus::Ongoing { .. }));
+        assert_eq!(decoded.metadata().save_kind, SaveKind::Recovery);
+        assert_eq!(decoded.metadata().session_id, "match-opening");
+        assert_eq!(
+            decoded.shell_state().pending_promotion,
+            Some(PendingPromotionSnapshot { from: e7, to: e8 })
+        );
+        assert_eq!(
+            decoded.shell_state().claimed_draw,
+            Some(ClaimedDrawSnapshot::ThreefoldRepetition)
+        );
     }
 }

```

**Documentation:**

```diff
--- a/crates/chess_persistence/src/lib.rs
+++ b/crates/chess_persistence/src/lib.rs
@@ -1,4 +1,7 @@
+//! Public persistence surface for the game shell.
+//! Consumers restore domain snapshots plus shell metadata through this crate instead of reaching into file-format details. (ref: DL-002) (ref: DL-004)
+
 pub mod snapshot;
 pub mod store;
 
@@ -16,6 +19,9 @@ mod tests {
         SnapshotMetadata, SnapshotShellState,
     };
 
+    // Pending promotion and claimed-draw metadata roundtrip here because recovery correctness depends
+    // on restoring legality-critical shell state alongside the domain snapshot. (ref: DL-004)
     #[test]
     fn snapshot_roundtrips_domain_state_and_shell_metadata() {
         let start = GameState::starting_position();

```


**CC-M-001-004** (crates/chess_persistence/Cargo.toml) - implements CI-M-001-004

**Code:**

```diff
--- a/crates/chess_persistence/Cargo.toml
+++ b/crates/chess_persistence/Cargo.toml
@@ -6,8 +6,11 @@
 
 [dependencies]
 chess_core = { path = "../chess_core" }
+directories = "5.0.1"
 serde.workspace = true
 serde_json.workspace = true
+tempfile = "3.15.0"
+time = { version = "0.3.39", features = ["formatting"] }
 
 [lints]
 workspace = true

```

**Documentation:**

```diff
--- a/crates/chess_persistence/Cargo.toml
+++ b/crates/chess_persistence/Cargo.toml
@@ -6,8 +6,9 @@
 
 [dependencies]
 chess_core = { path = "../chess_core" }
+# directories, tempfile, and time back the repository with platform roots, atomic writes, and stable timestamps. (ref: DL-002) (ref: DL-008)
 directories = "5.0.1"
 serde.workspace = true
 serde_json.workspace = true
 tempfile = "3.15.0"

```


**CC-M-001-005** (crates/chess_persistence/tests/session_store.rs) - implements CI-M-001-005

**Code:**

```diff
--- a/crates/chess_persistence/tests/session_store.rs
+++ b/crates/chess_persistence/tests/session_store.rs
@@ -0,0 +1,136 @@
+use chess_core::{GameState, Move, Square};
+use tempfile::tempdir;
+
+use chess_persistence::{
+    DisplayMode, GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind,
+    SessionStore, ShellSettings, SnapshotMetadata, SnapshotShellState,
+};
+
+fn sample_snapshot(label: &str) -> GameSnapshot {
+    let game_state =
+        GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("fixture FEN should parse");
+    let from = Square::from_algebraic("e7").expect("valid square");
+    let to = Square::from_algebraic("e8").expect("valid square");
+
+    GameSnapshot::from_parts(
+        game_state,
+        SnapshotMetadata {
+            label: label.to_string(),
+            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
+            updated_at_utc: None,
+            notes: Some(String::from("integration coverage")),
+            save_kind: SaveKind::Manual,
+            session_id: label.to_ascii_lowercase().replace(' ', "-"),
+            recovery_key: None,
+        },
+        SnapshotShellState {
+            selected_square: Some(from),
+            pending_promotion: Some(PendingPromotionSnapshot { from, to }),
+            last_move: Some(Move::new(from, to)),
+            claimed_draw: None,
+            dirty_recovery: true,
+        },
+    )
+}
+
+#[test]
+fn manual_save_list_load_delete_roundtrip() {
+    let root = tempdir().expect("temporary directory should be created");
+    let store = SessionStore::new(root.path());
+
+    let saved = store
+        .save_manual(sample_snapshot("Opening Save"))
+        .expect("manual save should succeed");
+    let list = store
+        .list_manual_saves()
+        .expect("manual save listing should succeed");
+    let loaded = store
+        .load_manual(&saved.slot_id)
+        .expect("manual save load should succeed");
+
+    assert_eq!(list.len(), 1);
+    assert_eq!(list[0].slot_id, saved.slot_id);
+    assert_eq!(loaded.metadata().label, "Opening Save");
+    assert!(loaded.shell_state().pending_promotion.is_some());
+
+    store
+        .delete_manual(&saved.slot_id)
+        .expect("manual save delete should succeed");
+    assert!(
+        store
+            .list_manual_saves()
+            .expect("manual save listing should succeed")
+            .is_empty()
+    );
+}
+
+#[test]
+fn recovery_record_and_settings_roundtrip() {
+    let root = tempdir().expect("temporary directory should be created");
+    let store = SessionStore::new(root.path());
+    let settings = ShellSettings {
+        recovery_policy: RecoveryStartupPolicy::Resume,
+        display_mode: DisplayMode::Fullscreen,
+        ..ShellSettings::default()
+    };
+
+    let saved_recovery = store
+        .store_recovery(sample_snapshot("Recovery Save"))
+        .expect("recovery save should succeed");
+    store
+        .save_settings(&settings)
+        .expect("settings write should succeed");
+
+    let loaded_recovery = store
+        .load_recovery()
+        .expect("recovery load should succeed")
+        .expect("recovery snapshot should exist");
+    let loaded_settings = store.load_settings().expect("settings load should succeed");
+
+    assert_eq!(saved_recovery.save_kind, SaveKind::Recovery);
+    assert_eq!(loaded_recovery.metadata().save_kind, SaveKind::Recovery);
+    assert_eq!(loaded_settings.display_mode, DisplayMode::Fullscreen);
+    assert_eq!(
+        loaded_settings.recovery_policy,
+        RecoveryStartupPolicy::Resume
+    );
+
+    store
+        .clear_recovery()
+        .expect("recovery clear should succeed");
+    assert!(
+        store
+            .load_recovery()
+            .expect("recovery load should succeed")
+            .is_none()
+    );
+}
+
+#[test]
+fn duplicate_labels_allocate_distinct_manual_slots() {
+    let root = tempdir().expect("temporary directory should be created");
+    let store = SessionStore::new(root.path());
+
+    let first = store
+        .save_manual(sample_snapshot("Opening Save"))
+        .expect("first manual save should succeed");
+    let second = store
+        .save_manual(sample_snapshot("Opening Save"))
+        .expect("second manual save should succeed");
+
+    assert_eq!(first.slot_id, "opening-save");
+    assert_eq!(second.slot_id, "opening-save-2");
+    assert_eq!(
+        store
+            .list_manual_saves()
+            .expect("manual save listing should succeed")
+            .len(),
+        2
+    );
+}
+
+#[test]
+fn default_root_uses_platform_app_data_location() {
+    let root = SessionStore::default_root().expect("platform default root should resolve");
+    assert!(root.to_string_lossy().contains("3d-chess"));
+}

```

**Documentation:**

```diff
--- a/crates/chess_persistence/tests/session_store.rs
+++ b/crates/chess_persistence/tests/session_store.rs
@@ -1,5 +1,8 @@
+//! Integration coverage for the file-backed session repository.
+//! These tests lock down user-visible save history, dedicated recovery behavior, and platform-path policy. (ref: DL-003) (ref: DL-008)
+
 use chess_core::{GameState, Move, Square};
 use chess_persistence::{
     GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore,
@@ -47,6 +50,8 @@ fn sample_snapshot(label: &str) -> GameSnapshot {
     )
 }
 
+// Manual history stays distinct from recovery so save slots remain user-controlled. (ref: DL-003)
 #[test]
 fn manual_save_list_load_delete_roundtrip() {
     let root = tempdir().expect("temporary directory should be created");
@@ -78,6 +83,8 @@ fn manual_save_list_load_delete_roundtrip() {
     assert_eq!(store.list_manual_saves().expect("listing saves should succeed"), vec![]);
 }
 
+// Recovery and settings share the repository root because startup policy and autosave live in the same shell contract. (ref: DL-005) (ref: DL-008)
 #[test]
 fn recovery_record_and_settings_roundtrip() {
     let root = tempdir().expect("temporary directory should be created");
@@ -120,6 +127,8 @@ fn recovery_record_and_settings_roundtrip() {
     assert_eq!(loaded_settings, settings);
 }
 
+// Duplicate labels resolve to separate slot IDs so deliberate saves do not overwrite each other. (ref: DL-003)
 #[test]
 fn duplicate_labels_allocate_distinct_manual_slots() {
     let root = tempdir().expect("temporary directory should be created");

```


**CC-M-001-006** (Cargo.lock) - implements CI-M-001-006

**Code:**

```diff
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1738,8 +1738,11 @@
 version = "0.1.0"
 dependencies = [
  "chess_core",
+ "directories",
  "serde",
  "serde_json",
+ "tempfile",
+ "time",
 ]
 
 [[package]]
@@ -2050,6 +2053,15 @@
 checksum = "d7a1e2f27636f116493b8b860f5546edb47c8d8f8ea73e1d2a20be88e28d1fea"
 
 [[package]]
+name = "deranged"
+version = "0.5.8"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "7cd812cc2bc1d69d4764bd80df88b4317eaef9e773c75226407d9bc0876b211c"
+dependencies = [
+ "powerfmt",
+]
+
+[[package]]
 name = "derive_more"
 version = "2.1.1"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -2073,6 +2085,27 @@
 ]
 
 [[package]]
+name = "directories"
+version = "5.0.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "9a49173b84e034382284f27f1af4dcbbd231ffa358c0fe316541a7337f376a35"
+dependencies = [
+ "dirs-sys",
+]
+
+[[package]]
+name = "dirs-sys"
+version = "0.4.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "520f05a5cbd335fae5a99ff7a6ab8627577660ee5cfd6a94a6a929b52ff0321c"
+dependencies = [
+ "libc",
+ "option-ext",
+ "redox_users",
+ "windows-sys 0.48.0",
+]
+
+[[package]]
 name = "dispatch"
 version = "0.2.0"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -2419,6 +2452,17 @@
 
 [[package]]
 name = "getrandom"
+version = "0.2.17"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0"
+dependencies = [
+ "cfg-if",
+ "libc",
+ "wasi",
+]
+
+[[package]]
+name = "getrandom"
 version = "0.3.4"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd"
@@ -3209,6 +3253,12 @@
 ]
 
 [[package]]
+name = "num-conv"
+version = "0.2.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "cf97ec579c3c42f953ef76dbf8d55ac91fb219dde70e49aa4a6b7d74e9919050"
+
+[[package]]
 name = "num-derive"
 version = "0.4.2"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -3541,6 +3591,12 @@
 checksum = "9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50"
 
 [[package]]
+name = "option-ext"
+version = "0.2.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "04744f49eae99ab78e0d5c0b603ab218f515ea8cfe5a456d7629ad883a3b6e7d"
+
+[[package]]
 name = "orbclient"
 version = "0.3.51"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -3714,6 +3770,12 @@
 ]
 
 [[package]]
+name = "powerfmt"
+version = "0.2.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391"
+
+[[package]]
 name = "pp-rs"
 version = "0.2.1"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -3914,6 +3976,17 @@
 ]
 
 [[package]]
+name = "redox_users"
+version = "0.4.6"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "ba009ff324d1fc1b900bd1fdb31564febe58a8ccc8a6fdbb93b543d33b13ca43"
+dependencies = [
+ "getrandom 0.2.17",
+ "libredox",
+ "thiserror 1.0.69",
+]
+
+[[package]]
 name = "regex"
 version = "1.12.3"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -4346,6 +4419,19 @@
 ]
 
 [[package]]
+name = "tempfile"
+version = "3.27.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd"
+dependencies = [
+ "fastrand",
+ "getrandom 0.4.2",
+ "once_cell",
+ "rustix 1.1.4",
+ "windows-sys 0.61.2",
+]
+
+[[package]]
 name = "termcolor"
 version = "1.4.1"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -4404,6 +4490,37 @@
 ]
 
 [[package]]
+name = "time"
+version = "0.3.47"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "743bd48c283afc0388f9b8827b976905fb217ad9e647fae3a379a9283c4def2c"
+dependencies = [
+ "deranged",
+ "itoa",
+ "num-conv",
+ "powerfmt",
+ "serde_core",
+ "time-core",
+ "time-macros",
+]
+
+[[package]]
+name = "time-core"
+version = "0.1.8"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "7694e1cfe791f8d31026952abf09c69ca6f6fa4e1a1229e18988f06a04a12dca"
+
+[[package]]
+name = "time-macros"
+version = "0.2.27"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "2e70e4c5a0e0a8a4823ad65dfe1a6930e4f4d756dcd9dd7939022b5e8c501215"
+dependencies = [
+ "num-conv",
+ "time-core",
+]
+
+[[package]]
 name = "tiny-skia"
 version = "0.11.4"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -4726,6 +4843,12 @@
 ]
 
 [[package]]
+name = "wasi"
+version = "0.11.1+wasi-snapshot-preview1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"
+
+[[package]]
 name = "wasip2"
 version = "1.0.2+wasi-0.2.9"
 source = "registry+https://github.com/rust-lang/crates.io-index"
@@ -5428,6 +5551,15 @@
 
 [[package]]
 name = "windows-sys"
+version = "0.48.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "677d2418bec65e3338edb076e806bc1ec15693c5d0104683f2efe857f61056a9"
+dependencies = [
+ "windows-targets 0.48.5",
+]
+
+[[package]]
+name = "windows-sys"
 version = "0.52.0"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d"
@@ -5470,6 +5602,21 @@
 
 [[package]]
 name = "windows-targets"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "9a2fa6e2155d7247be68c096456083145c183cbbbc2764150dda45a87197940c"
+dependencies = [
+ "windows_aarch64_gnullvm 0.48.5",
+ "windows_aarch64_msvc 0.48.5",
+ "windows_i686_gnu 0.48.5",
+ "windows_i686_msvc 0.48.5",
+ "windows_x86_64_gnu 0.48.5",
+ "windows_x86_64_gnullvm 0.48.5",
+ "windows_x86_64_msvc 0.48.5",
+]
+
+[[package]]
+name = "windows-targets"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973"
@@ -5510,6 +5657,12 @@
 
 [[package]]
 name = "windows_aarch64_gnullvm"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "2b38e32f0abccf9987a4e3079dfb67dcd799fb61361e53e2882c3cbaf0d905d8"
+
+[[package]]
+name = "windows_aarch64_gnullvm"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3"
@@ -5522,6 +5675,12 @@
 
 [[package]]
 name = "windows_aarch64_msvc"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc"
+
+[[package]]
+name = "windows_aarch64_msvc"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469"
@@ -5534,6 +5693,12 @@
 
 [[package]]
 name = "windows_i686_gnu"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "a75915e7def60c94dcef72200b9a8e58e5091744960da64ec734a6c6e9b3743e"
+
+[[package]]
+name = "windows_i686_gnu"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b"
@@ -5552,6 +5717,12 @@
 
 [[package]]
 name = "windows_i686_msvc"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "8f55c233f70c4b27f66c523580f78f1004e8b5a8b659e05a4eb49d4166cca406"
+
+[[package]]
+name = "windows_i686_msvc"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66"
@@ -5564,6 +5735,12 @@
 
 [[package]]
 name = "windows_x86_64_gnu"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "53d40abd2583d23e4718fddf1ebec84dbff8381c07cae67ff7768bbf19c6718e"
+
+[[package]]
+name = "windows_x86_64_gnu"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78"
@@ -5576,6 +5753,12 @@
 
 [[package]]
 name = "windows_x86_64_gnullvm"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "0b7b52767868a23d5bab768e390dc5f5c55825b6d30b86c844ff2dc7414044cc"
+
+[[package]]
+name = "windows_x86_64_gnullvm"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d"
@@ -5588,6 +5771,12 @@
 
 [[package]]
 name = "windows_x86_64_msvc"
+version = "0.48.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "ed94fce61571a4006852b7389a063ab983c02eb1bb37b47f8272ce92d06d9538"
+
+[[package]]
+name = "windows_x86_64_msvc"
 version = "0.52.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec"
```

**Documentation:**

```diff
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1738,8 +1738,9 @@
 version = "0.1.0"
+# Session-store dependencies cover platform app-data resolution, atomic write staging, and RFC3339 timestamps. (ref: DL-002) (ref: DL-008)
 dependencies = [
  "chess_core",
  "directories",
  "serde",
  "serde_json",
  "tempfile",
  "time",
 ]

```


**CC-M-001-007** (crates/chess_persistence/README.md)

**Documentation:**

```diff
--- /dev/null
+++ b/crates/chess_persistence/README.md
@@ -0,0 +1,22 @@
+# chess_persistence
+
+Saved-session repository and snapshot contract for the local game shell.
+
+## Architecture
+
+- `GameSnapshot` stores authoritative `chess_core` state plus the shell metadata required to resume legal interaction.
+- `SessionStore` owns filesystem layout, atomic writes, and the default app-data root so Bevy systems only exchange snapshots. (ref: DL-002) (ref: DL-008)
+
+## Invariants
+
+- Domain snapshots remain the restore boundary; ECS world state is never persisted. (ref: DL-004)
+- Manual saves and interrupted-session recovery stay separate user concepts with separate overwrite rules. (ref: DL-003)
+- The shipped settings contract only covers startup recovery behavior, destructive confirmations, and display mode. (ref: DL-005)
+
+## Layout
+
+- `saves/` holds manual save slots.
+- `recovery/current.json` holds the single interrupted-session record.
+- `settings.json` holds shell settings.
+
+## Testing
+
+- Repository tests inject temp roots so platform-path behavior stays deterministic.

```


### Milestone 2: Product Shell Save-Load UX and Recovery

**M3 Shell State Flow**

[Diagram pending Technical Writer rendering: DIAG-002]

**Files**: Cargo.lock, crates/game_app/Cargo.toml, crates/game_app/src/app.rs, crates/game_app/src/lib.rs, crates/game_app/src/match_state.rs, crates/game_app/src/plugins/mod.rs, crates/game_app/src/plugins/app_shell.rs, crates/game_app/src/plugins/input.rs, crates/game_app/src/plugins/move_feedback.rs, crates/game_app/src/plugins/scaffold.rs, crates/game_app/src/plugins/menu.rs, crates/game_app/src/plugins/save_load.rs, crates/game_app/tests/match_state_flow.rs, crates/game_app/tests/save_load_flow.rs

**Flags**: shell-flow, save-load, recovery

**Requirements**:

- Turn the M2 playable loop into a complete local product shell with main-menu setup
- in-match pause overlays
- and result flows
- Route new load and resume actions through explicit launch intent and recovery state instead of hard-resetting MatchLoading
- Expose shipped save-load UX and minimal settings backed by chess_persistence while preserving chess_core authority and a coarse top-level app state

**Acceptance Criteria**:

- Users can start
- pause via the in-match setup overlay
- save
- load
- resume
- and finish local matches through shipped UI flows without adding extra top-level app screens
- Interrupted-session recovery restores the expected match after restart or simulated restart through MatchLoading
- Integration tests initialize the full shell plugin stack and cover MatchLoading branches plus keyboard-driven pause and quick-save behavior without regressing chess_core authority

**Tests**:

- integration: MatchLoading load and startup-resume branches with the full shell plugin stack
- integration: keyboard pause overlay and F5 quick-save recovery roundtrip

#### Code Intent

- **CI-M-002-001** `crates/game_app/src/match_state.rs::session lifecycle bridge`: Add launch-intent session-summary and persistence-conversion helpers so MatchSession can hydrate from saved-session snapshots and report dirty recovery state without taking ownership of repository I/O. (refs: DL-001, DL-004, DL-007)
- **CI-M-002-002** `crates/game_app/src/app.rs::build_app`: Register concrete menu and save-load plugins plus startup resources that surface recovery availability and drive the coarse M3 screen flow. (refs: DL-001, DL-007)
- **CI-M-002-003** `crates/game_app/src/plugins/mod.rs::plugin exports`: Promote menu and save-load seams into concrete plugins while leaving AI and audio placeholder boundaries intact for later milestones. (refs: DL-007)
- **CI-M-002-004** `crates/game_app/src/plugins/scaffold.rs::placeholder seams`: Shrink scaffold responsibilities so only deferred AI and audio shells remain after menu and save-load plugins become live M3 code paths. (refs: DL-007)
- **CI-M-002-005** `crates/game_app/src/plugins/menu.rs::MenuPlugin`: Own main-menu setup, in-match pause overlays, load-list selection, and settings view state so shell menus stay orthogonal to gameplay and persistence execution. (refs: DL-001, DL-005, DL-007)
- **CI-M-002-006** `crates/game_app/src/plugins/save_load.rs::SaveLoadPlugin`: Connect menu actions and recovery policy to chess_persistence so the app can create manual saves restore sessions manage recovery records and surface overwrite or error states through Bevy resources and events. (refs: DL-002, DL-003, DL-007)
- **CI-M-002-007** `crates/game_app/src/plugins/app_shell.rs::shell UI and state transitions`: Expand shell presentation to cover main-menu setup pause result resume load and confirmation flows while consuming launch intents and modal resources instead of resetting matches inline. (refs: DL-001, DL-005, DL-007)
- **CI-M-002-008** `crates/game_app/src/plugins/input.rs::match input routing`: Route Escape and related in-match shell controls through paused and modal flows without bypassing pending-promotion or save-confirmation guards. (refs: DL-001, DL-007)
- **CI-M-002-009** `crates/game_app/src/plugins/move_feedback.rs::status and feedback surfaces`: Surface save-load recovery and error feedback alongside the existing turn draw and promotion messaging so shell actions remain visible without changing chess rules ownership. (refs: DL-004, DL-007)
- **CI-M-002-010** `crates/game_app/src/lib.rs::test-facing exports`: Re-export the M3 shell resources and plugins needed by integration tests to drive save-load and recovery flows through the public crate surface. (refs: DL-001, DL-007)
- **CI-M-002-011** `crates/game_app/Cargo.toml::crate manifest`: Add only the game_app dependencies and dev-dependencies required for shell persistence integration and real flow tests. (refs: DL-002, DL-007)
- **CI-M-002-012** `crates/game_app/tests/match_state_flow.rs::screen-state coverage`: Initialize the full shell plugin stack in integration tests and cover manual-load, startup-resume, and keyboard-pause branches through MatchLoading without introducing extra top-level app states. (refs: DL-002, DL-005)
- **CI-M-002-013** `crates/game_app/tests/save_load_flow.rs::end-to-end shell persistence coverage`: Drive keyboard quick-save, manual-load, and restarted recovery flows against a temporary save root while asserting the in-match setup overlay stays orthogonal to the coarse AppScreenState. (refs: DL-002, DL-005)
- **CI-M-002-014** `Cargo.lock::workspace lockfile`: Refresh Cargo.lock only for the game_app test dependency additions introduced in M2, reusing the persistence dependency resolution already captured in M1. (refs: DL-002, DL-007)

#### Code Changes

**CC-M-002-001** (crates/game_app/src/match_state.rs) - implements CI-M-002-001

**Code:**

```diff
--- a/crates/game_app/src/match_state.rs
+++ b/crates/game_app/src/match_state.rs
@@ -1,5 +1,29 @@
 use bevy::prelude::Resource;
 use chess_core::{DrawAvailability, GameState, GameStatus, Move, MoveError, Piece, Square};
+use chess_persistence::{
+    ClaimedDrawSnapshot, GameSnapshot, PendingPromotionSnapshot, SnapshotMetadata,
+    SnapshotShellState,
+};
+
+#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
+pub enum MatchLaunchIntent {
+    #[default]
+    NewLocalMatch,
+    LoadManual,
+    ResumeRecovery,
+    Rematch,
+}
+
+#[derive(Resource, Debug, Clone, Default)]
+pub struct PendingLoadedSnapshot(pub Option<GameSnapshot>);
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct MatchSessionSummary {
+    pub status: GameStatus,
+    pub last_move: Option<Move>,
+    pub pending_promotion: bool,
+    pub dirty_recovery: bool,
+}
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum ClaimedDrawReason {
@@ -7,8 +31,8 @@
     FiftyMoveRule,
 }
 
-// MatchSession is the sole Bevy-facing bridge to chess_core during M2 so presentation never becomes the rules authority.
-// Pending promotion and claimable-draw state live here because they are interaction concerns layered on top of chess_core legality.
+// MatchSession stays the only Bevy-facing bridge to chess_core.
+// M3 adds snapshot conversion so save/load never turns Bevy state into the authority.
 #[derive(Resource, Debug, Clone)]
 pub struct MatchSession {
     pub game_state: GameState,
@@ -16,6 +40,7 @@
     pub pending_promotion_move: Option<Move>,
     pub last_move: Option<Move>,
     pub claimed_draw: Option<ClaimedDrawReason>,
+    dirty_recovery: bool,
 }
 
 impl MatchSession {
@@ -27,11 +52,67 @@
             pending_promotion_move: None,
             last_move: None,
             claimed_draw: None,
+            dirty_recovery: true,
         }
     }
 
     pub fn reset_for_local_match(&mut self) {
         *self = Self::start_local_match();
+    }
+
+    #[must_use]
+    pub fn restore_from_snapshot(snapshot: &GameSnapshot) -> Self {
+        let claimed_draw = snapshot.shell_state().claimed_draw.map(|reason| match reason {
+            ClaimedDrawSnapshot::ThreefoldRepetition => ClaimedDrawReason::ThreefoldRepetition,
+            ClaimedDrawSnapshot::FiftyMoveRule => ClaimedDrawReason::FiftyMoveRule,
+        });
+
+        Self {
+            game_state: snapshot.restore_game_state(),
+            selected_square: snapshot.shell_state().selected_square,
+            pending_promotion_move: snapshot
+                .shell_state()
+                .pending_promotion
+                .map(|promotion| Move::new(promotion.from, promotion.to)),
+            last_move: snapshot.shell_state().last_move,
+            claimed_draw,
+            dirty_recovery: snapshot.shell_state().dirty_recovery,
+        }
+    }
+
+    #[must_use]
+    pub fn to_snapshot(&self, metadata: SnapshotMetadata) -> GameSnapshot {
+        let claimed_draw = self.claimed_draw.map(|reason| match reason {
+            ClaimedDrawReason::ThreefoldRepetition => ClaimedDrawSnapshot::ThreefoldRepetition,
+            ClaimedDrawReason::FiftyMoveRule => ClaimedDrawSnapshot::FiftyMoveRule,
+        });
+
+        GameSnapshot::from_parts(
+            self.game_state.clone(),
+            metadata,
+            SnapshotShellState {
+                selected_square: self.selected_square,
+                pending_promotion: self.pending_promotion_move.map(|promotion| {
+                    PendingPromotionSnapshot {
+                        from: promotion.from(),
+                        to: promotion.to(),
+                    }
+                }),
+                last_move: self.last_move,
+                claimed_draw,
+                dirty_recovery: self.dirty_recovery,
+            },
+        )
+    }
+
+    #[must_use]
+    pub fn summary(&self) -> MatchSessionSummary {
+        MatchSessionSummary {
+            status: self.status(),
+            last_move: self.last_move,
+            pending_promotion: self.pending_promotion_move.is_some(),
+            dirty_recovery: self.dirty_recovery,
+        }
     }
 
     #[must_use]
@@ -44,6 +125,7 @@
         self.last_move = None;
         self.claimed_draw = None;
         self.clear_interaction();
+        self.mark_recovery_dirty();
     }
 
     #[must_use]
@@ -86,6 +168,7 @@
         self.last_move = Some(candidate);
         self.claimed_draw = None;
         self.clear_interaction();
+        self.mark_recovery_dirty();
         Ok(())
     }
 
@@ -97,6 +180,19 @@
     #[must_use]
     pub fn is_finished(&self) -> bool {
         self.claimed_draw.is_some() || self.status().is_finished()
+    }
+
+    #[must_use]
+    pub fn is_recovery_dirty(&self) -> bool {
+        self.dirty_recovery
+    }
+
+    pub fn mark_recovery_dirty(&mut self) {
+        self.dirty_recovery = true;
+    }
+
+    pub fn mark_recovery_persisted(&mut self) {
+        self.dirty_recovery = false;
     }
 
     pub fn claim_draw(&mut self) -> bool {
@@ -115,6 +211,7 @@
 
         self.claimed_draw = Some(reason);
         self.clear_interaction();
+        self.mark_recovery_dirty();
         true
     }
 }

```

**Documentation:**

```diff
--- a/crates/game_app/src/match_state.rs
+++ b/crates/game_app/src/match_state.rs
@@ -1,5 +1,8 @@
+//! Bevy-facing match bridge for local play, load, and recovery flows.
+//! Snapshot conversion keeps `chess_core` authoritative while the shell restores only the interaction state it needs. (ref: DL-001) (ref: DL-004)
+
 use bevy::prelude::Resource;
 use chess_core::{DrawAvailability, GameState, GameStatus, Move, MoveError, Piece, Square};
 use chess_persistence::{
@@ -6,6 +9,8 @@ use chess_persistence::{
     SnapshotShellState,
 };
 
+/// Describes how MatchLoading should hydrate the next match without exploding top-level screen routing. (ref: DL-001)
 #[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
 pub enum MatchLaunchIntent {
     #[default]
@@ -52,6 +57,9 @@ impl MatchSession {
             dirty_recovery: true,
         }
     }
 
+    /// Restores a playable session from persisted domain and shell metadata.
+    /// Bevy interaction state rebuilds from the snapshot instead of acting as a second source of truth. (ref: DL-004)
     #[must_use]
     pub fn restore_from_snapshot(snapshot: &GameSnapshot) -> Self {
         let claimed_draw = snapshot.shell_state().claimed_draw.map(|reason| match reason {
@@ -77,6 +85,9 @@ impl MatchSession {
         }
     }
 
+    /// Produces the persisted session contract that save/load plugins hand to the repository boundary. (ref: DL-002) (ref: DL-004)
     #[must_use]
     pub fn to_snapshot(&self, metadata: SnapshotMetadata) -> GameSnapshot {
         let claimed_draw = self.claimed_draw.map(|reason| match reason {
@@ -99,6 +110,8 @@ impl MatchSession {
         )
     }
 
+    /// Summarizes only shell-relevant facts so UI can render status without reaching through gameplay internals. (ref: DL-007)
     #[must_use]
     pub fn summary(&self) -> MatchSessionSummary {
         MatchSessionSummary {

```


**CC-M-002-002** (crates/game_app/src/app.rs) - implements CI-M-002-002

**Code:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@ -1,10 +1,11 @@
 use bevy::prelude::*;
 use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
 
-use crate::match_state::MatchSession;
+use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
 use crate::plugins::{
     AiMatchPlugin, AppShellPlugin, BoardScenePlugin, ChessAudioPlugin, MenuPlugin,
-    MoveFeedbackPlugin, PieceViewPlugin, SaveLoadPlugin, ShellInputPlugin,
+    MoveFeedbackPlugin, PieceViewPlugin, RecoveryBannerState, SaveLoadPlugin, SaveLoadState,
+    SaveRootOverride, ShellInputPlugin, ShellMenuState,
 };
 use crate::style::ShellTheme;
 
@@ -15,10 +16,8 @@ pub enum AppScreenState {
     #[default]
     Boot,
     MainMenu,
-    LocalSetup,
     MatchLoading,
     InMatch,
-    Paused,
     MatchResult,
 }
 
@@ -41,14 +40,21 @@ pub fn build_app() -> App {
         }))
         .init_state::<AppScreenState>()
         .insert_resource(MatchSession::start_local_match())
+        // Startup resources keep recovery visibility and launch intent coarse at the app root.
+        .insert_resource(MatchLaunchIntent::default())
+        .insert_resource(PendingLoadedSnapshot::default())
+        .insert_resource(ShellMenuState::default())
+        .insert_resource(RecoveryBannerState::default())
+        .insert_resource(SaveLoadState::default())
+        .insert_resource(SaveRootOverride::default())
         .add_plugins((
+            MenuPlugin,
+            SaveLoadPlugin,
             AppShellPlugin,
             BoardScenePlugin,
             PieceViewPlugin,
             ShellInputPlugin,
             MoveFeedbackPlugin,
-            MenuPlugin,
-            SaveLoadPlugin,
             AiMatchPlugin,
             ChessAudioPlugin,
         ));

```

**Documentation:**

```diff
--- a/crates/game_app/src/app.rs
+++ b/crates/game_app/src/app.rs
@@ -26,6 +26,9 @@ pub enum AppScreenState {
     MatchResult,
 }
 
+/// Builds the coarse screen-state shell and keeps menu/save-load concerns in orthogonal resources.
+/// Modal flow stays outside the top-level route enum so the local shell grows without routing sprawl. (ref: DL-001) (ref: DL-007)
 pub fn build_app() -> App {
     App::new()
         .add_plugins(DefaultPlugins.set(WindowPlugin {
@@ -41,6 +44,7 @@ pub fn build_app() -> App {
         .init_state::<AppScreenState>()
         .insert_resource(MatchSession::start_local_match())
+        // These resources carry launch intent and modal shell state across the small set of top-level routes. (ref: DL-001)
         .insert_resource(MatchLaunchIntent::default())
         .insert_resource(PendingLoadedSnapshot::default())
         .insert_resource(ShellMenuState::default())

```


**CC-M-002-003** (crates/game_app/src/plugins/mod.rs) - implements CI-M-002-003

**Code:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,13 +1,23 @@
 mod app_shell;
 mod board_scene;
 mod input;
+mod menu;
 mod move_feedback;
 mod piece_view;
+mod save_load;
 mod scaffold;
 
 pub use app_shell::AppShellPlugin;
 pub use board_scene::{BoardScenePlugin, BoardSquareVisual};
 pub use input::ShellInputPlugin;
+pub use menu::{
+    ConfirmationKind, MenuAction, MenuPanel, MenuPlugin, RecoveryBannerState,
+    ShellMenuState,
+};
 pub use move_feedback::MoveFeedbackPlugin;
 pub use piece_view::{PieceViewPlugin, PieceVisual};
-pub use scaffold::{AiMatchPlugin, ChessAudioPlugin, MenuPlugin, SaveLoadPlugin};
+pub use save_load::{
+    SaveLoadPlugin, SaveLoadRequest, SaveLoadState, SaveRootOverride,
+    SessionStoreResource,
+};
+pub use scaffold::{AiMatchPlugin, ChessAudioPlugin};

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/mod.rs
+++ b/crates/game_app/src/plugins/mod.rs
@@ -1,3 +1,6 @@
+//! Shell plugins split by concern.
+//! Menu state, persistence I/O, and presentation stay separate so the app shell does not collapse into a single coordinator. (ref: DL-007)
+
 mod app_shell;
 mod board_scene;
 mod input;
@@ -8,6 +11,8 @@ mod scaffold;
 pub use app_shell::AppShellPlugin;
 pub use board_scene::{BoardScenePlugin, BoardSquareVisual};
 pub use input::ShellInputPlugin;
+// Re-export modal shell primitives so tests and top-level app wiring share the same coarse routing model. (ref: DL-001) (ref: DL-007)
 pub use menu::{
     ConfirmationKind, MenuAction, MenuPanel, MenuPlugin, RecoveryBannerState,
     ShellMenuState,

```


**CC-M-002-004** (crates/game_app/src/plugins/scaffold.rs) - implements CI-M-002-004

**Code:**

```diff
--- a/crates/game_app/src/plugins/scaffold.rs
+++ b/crates/game_app/src/plugins/scaffold.rs
@@ -1,17 +1,7 @@
 use bevy::prelude::*;
 
-pub struct MenuPlugin;
-pub struct SaveLoadPlugin;
 pub struct AiMatchPlugin;
 pub struct ChessAudioPlugin;
-
-impl Plugin for MenuPlugin {
-    fn build(&self, _app: &mut App) {}
-}
-
-impl Plugin for SaveLoadPlugin {
-    fn build(&self, _app: &mut App) {}
-}
 
 impl Plugin for AiMatchPlugin {
     fn build(&self, _app: &mut App) {}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/scaffold.rs
+++ b/crates/game_app/src/plugins/scaffold.rs
@@ -1,6 +1,8 @@
 use bevy::prelude::*;
 
+/// Keeps the AI lifecycle seam explicit without pulling engine orchestration into the shipped shell plugin set. (ref: DL-007)
 pub struct AiMatchPlugin;
+/// Keeps audio wiring explicit while the local shell focuses on save/load and recovery behavior. (ref: DL-007)
 pub struct ChessAudioPlugin;
 
 impl Plugin for AiMatchPlugin {

```


**CC-M-002-005** (crates/game_app/src/plugins/menu.rs) - implements CI-M-002-005

**Code:**

```diff
--- a/crates/game_app/src/plugins/menu.rs
+++ b/crates/game_app/src/plugins/menu.rs
@@ -0,0 +1,151 @@
+use bevy::prelude::*;
+
+use crate::app::AppScreenState;
+use crate::match_state::MatchLaunchIntent;
+
+pub struct MenuPlugin;
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+pub enum MenuPanel {
+    #[default]
+    Home,
+    Setup,
+    LoadList,
+    Settings,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
+pub enum MenuContext {
+    #[default]
+    MainMenu,
+    InMatchOverlay,
+}
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum ConfirmationKind {
+    AbandonMatch,
+    DeleteSave,
+    OverwriteSave,
+}
+
+#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
+pub struct RecoveryBannerState {
+    pub available: bool,
+    pub dirty: bool,
+    pub label: Option<String>,
+}
+
+#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
+pub struct ShellMenuState {
+    pub panel: MenuPanel,
+    pub context: MenuContext,
+    pub confirmation: Option<ConfirmationKind>,
+    pub selected_save: Option<String>,
+    pub status_line: Option<String>,
+}
+
+#[derive(Event, Debug, Clone, PartialEq, Eq)]
+pub enum MenuAction {
+    OpenSetup,
+    OpenLoadList,
+    OpenSettings,
+    BackToSetup,
+    StartNewMatch,
+    Rematch,
+    PauseMatch,
+    ResumeMatch,
+    ReturnToMenu,
+    SelectSave(String),
+    RequestConfirmation(ConfirmationKind),
+    CancelModal,
+}
+
+impl Plugin for MenuPlugin {
+    fn build(&self, app: &mut App) {
+        app.init_resource::<ShellMenuState>()
+            .init_resource::<RecoveryBannerState>()
+            .add_event::<MenuAction>()
+            .add_systems(Update, sync_menu_panel_from_screen)
+            .add_systems(Update, apply_menu_actions);
+    }
+}
+
+fn sync_menu_panel_from_screen(
+    state: Res<State<AppScreenState>>,
+    mut menu_state: ResMut<ShellMenuState>,
+) {
+    if !state.is_changed() {
+        return;
+    }
+
+    match state.get() {
+        AppScreenState::MainMenu => {
+            menu_state.context = MenuContext::MainMenu;
+            menu_state.panel = MenuPanel::Home;
+            menu_state.confirmation = None;
+        }
+        AppScreenState::MatchLoading => {
+            menu_state.confirmation = None;
+        }
+        AppScreenState::Boot | AppScreenState::InMatch | AppScreenState::MatchResult => {}
+    }
+}
+
+fn apply_menu_actions(
+    mut actions: EventReader<MenuAction>,
+    state: Res<State<AppScreenState>>,
+    mut menu_state: ResMut<ShellMenuState>,
+    mut launch_intent: ResMut<MatchLaunchIntent>,
+    mut next_state: ResMut<NextState<AppScreenState>>,
+) {
+    for action in actions.read() {
+        match action {
+            MenuAction::OpenSetup => {
+                menu_state.panel = MenuPanel::Setup;
+                menu_state.context = MenuContext::MainMenu;
+            }
+            MenuAction::OpenLoadList => {
+                menu_state.panel = MenuPanel::LoadList;
+            }
+            MenuAction::OpenSettings => {
+                menu_state.panel = MenuPanel::Settings;
+            }
+            MenuAction::BackToSetup => {
+                menu_state.panel = MenuPanel::Setup;
+            }
+            MenuAction::StartNewMatch => {
+                *launch_intent = MatchLaunchIntent::NewLocalMatch;
+                menu_state.context = MenuContext::MainMenu;
+                next_state.set(AppScreenState::MatchLoading);
+            }
+            MenuAction::Rematch => {
+                *launch_intent = MatchLaunchIntent::Rematch;
+                menu_state.context = MenuContext::MainMenu;
+                next_state.set(AppScreenState::MatchLoading);
+            }
+            MenuAction::PauseMatch => {
+                if *state.get() == AppScreenState::InMatch {
+                    menu_state.panel = MenuPanel::Setup;
+                    menu_state.context = MenuContext::InMatchOverlay;
+                    menu_state.confirmation = None;
+                }
+            }
+            MenuAction::ResumeMatch => {
+                menu_state.confirmation = None;
+                menu_state.context = MenuContext::MainMenu;
+            }
+            MenuAction::ReturnToMenu => {
+                menu_state.panel = MenuPanel::Home;
+                menu_state.context = MenuContext::MainMenu;
+                menu_state.confirmation = None;
+                next_state.set(AppScreenState::MainMenu);
+            }
+            MenuAction::SelectSave(slot_id) => {
+                menu_state.selected_save = Some(slot_id.clone());
+                menu_state.status_line = Some(format!("Selected save {slot_id}."));
+            }
+            MenuAction::RequestConfirmation(kind) => {
+                menu_state.confirmation = Some(*kind);
+            }
+            MenuAction::CancelModal => {
+                menu_state.confirmation = None;
+            }
+        }
+    }
+}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/menu.rs
+++ b/crates/game_app/src/plugins/menu.rs
@@ -1,4 +1,7 @@
+//! Modal shell state for main-menu, pause, and confirmation flows.
+//! Setup, load, and the shipped settings trio of startup recovery, destructive confirmations, and display mode remain overlays instead of separate top-level app routes. (ref: DL-001) (ref: DL-005) (ref: DL-007)
+
 use bevy::prelude::*;
 
 use crate::app::AppScreenState;
@@ -4,6 +7,8 @@ use crate::app::AppScreenState;
 use crate::match_state::MatchLaunchIntent;
 
+/// Owns menu events and modal shell resources while leaving snapshot I/O to the save-load plugin. (ref: DL-007)
 pub struct MenuPlugin;
@@ -29,6 +34,8 @@ pub struct RecoveryBannerState {
     pub label: Option<String>,
 }
 
+/// Captures transient shell state that can move between the main menu and the in-match overlay without changing the coarse app route. (ref: DL-001)
 #[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
 pub struct ShellMenuState {
     pub panel: MenuPanel,
@@ -61,6 +68,8 @@ impl Plugin for MenuPlugin {
     }
 }
 
+/// Applies user-facing shell actions to modal state and launch intent, leaving snapshot hydration to MatchLoading. (ref: DL-001)
 fn apply_menu_actions(
     mut actions: EventReader<MenuAction>,
     state: Res<State<AppScreenState>>,

```


**CC-M-002-006** (crates/game_app/src/plugins/save_load.rs) - implements CI-M-002-006

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -0,0 +1,310 @@
+use std::path::PathBuf;
+
+use bevy::prelude::*;
+use bevy::window::{MonitorSelection, PrimaryWindow, Window, WindowMode};
+use chess_persistence::{
+    DisplayMode, RecoveryStartupPolicy, SaveKind, SavedSessionSummary, SessionStore,
+    ShellSettings, SnapshotMetadata,
+};
+
+use super::menu::{MenuPanel, RecoveryBannerState, ShellMenuState};
+use crate::app::AppScreenState;
+use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
+
+pub struct SaveLoadPlugin;
+
+#[derive(Resource, Debug, Clone, Default)]
+pub struct SaveRootOverride(pub Option<PathBuf>);
+
+#[derive(Resource, Debug, Clone)]
+pub struct SessionStoreResource(pub SessionStore);
+
+#[derive(Resource, Debug, Clone, Default)]
+pub struct SaveLoadState {
+    pub manual_saves: Vec<SavedSessionSummary>,
+    pub recovery: Option<SavedSessionSummary>,
+    pub settings: ShellSettings,
+    pub last_message: Option<String>,
+    pub last_error: Option<String>,
+}
+
+#[derive(Event, Debug, Clone, PartialEq, Eq)]
+pub enum SaveLoadRequest {
+    RefreshIndex,
+    SaveManual { label: String, slot_id: Option<String> },
+    LoadManual { slot_id: String },
+    DeleteManual { slot_id: String },
+    ResumeRecovery,
+    ClearRecovery,
+    PersistSettings,
+}
+
+#[derive(Resource, Default)]
+struct StartupRecoveryHandled(bool);
+
+impl Plugin for SaveLoadPlugin {
+    fn build(&self, app: &mut App) {
+        app.init_resource::<SaveRootOverride>()
+            .init_resource::<SaveLoadState>()
+            .init_resource::<StartupRecoveryHandled>()
+            .add_event::<SaveLoadRequest>()
+            .add_systems(Startup, setup_store)
+            .add_systems(
+                Update,
+                (
+                    maybe_resume_recovery_on_startup,
+                    apply_display_mode_setting,
+                    handle_save_load_requests,
+                    autosave_active_match,
+                ),
+            )
+            .add_systems(OnEnter(AppScreenState::MatchResult), clear_result_recovery);
+    }
+}
+
+fn setup_store(
+    mut commands: Commands,
+    root_override: Res<SaveRootOverride>,
+    mut save_state: ResMut<SaveLoadState>,
+    mut recovery_banner: ResMut<RecoveryBannerState>,
+) {
+    let store = root_override
+        .0
+        .clone()
+        // Tests inject roots; runtime defaults to app-data so packaged builds never write into CWD.
+        .map(SessionStore::new)
+        .or_else(|| SessionStore::runtime().ok())
+        .unwrap_or_else(|| SessionStore::new(std::env::temp_dir().join("3d-chess")));
+
+    commands.insert_resource(SessionStoreResource(store));
+    refresh_store_index(&commands, &mut save_state, &mut recovery_banner);
+}
+
+fn maybe_resume_recovery_on_startup(
+    state: Res<State<AppScreenState>>,
+    mut handled: ResMut<StartupRecoveryHandled>,
+    store: Res<SessionStoreResource>,
+    save_state: Res<SaveLoadState>,
+    mut recovery_banner: ResMut<RecoveryBannerState>,
+    mut pending_snapshot: ResMut<PendingLoadedSnapshot>,
+    mut launch_intent: ResMut<MatchLaunchIntent>,
+    mut next_state: ResMut<NextState<AppScreenState>>,
+) {
+    if handled.0 || *state.get() != AppScreenState::MainMenu {
+        return;
+    }
+
+    handled.0 = true;
+    match save_state.settings.recovery_policy {
+        RecoveryStartupPolicy::Resume => {
+            if let Ok(Some(snapshot)) = store.0.load_recovery() {
+                pending_snapshot.0 = Some(snapshot);
+                *launch_intent = MatchLaunchIntent::ResumeRecovery;
+                next_state.set(AppScreenState::MatchLoading);
+            }
+        }
+        RecoveryStartupPolicy::Ignore => {
+            recovery_banner.available = false;
+        }
+        RecoveryStartupPolicy::Ask => {}
+    }
+}
+
+fn apply_display_mode_setting(
+    save_state: Res<SaveLoadState>,
+    mut windows: Query<&mut Window, With<PrimaryWindow>>,
+) {
+    if !save_state.is_changed() {
+        return;
+    }
+
+    let Ok(mut window) = windows.single_mut() else {
+        return;
+    };
+
+    window.mode = match save_state.settings.display_mode {
+        DisplayMode::Windowed => WindowMode::Windowed,
+        DisplayMode::Fullscreen => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
+    };
+}
+
+fn handle_save_load_requests(
+    mut requests: EventReader<SaveLoadRequest>,
+    store: Res<SessionStoreResource>,
+    match_session: Res<MatchSession>,
+    mut pending_snapshot: ResMut<PendingLoadedSnapshot>,
+    mut launch_intent: ResMut<MatchLaunchIntent>,
+    mut save_state: ResMut<SaveLoadState>,
+    mut recovery_banner: ResMut<RecoveryBannerState>,
+    mut menu_state: ResMut<ShellMenuState>,
+    mut next_state: ResMut<NextState<AppScreenState>>,
+) {
+    for request in requests.read() {
+        match request {
+            SaveLoadRequest::RefreshIndex => {
+                refresh_store_index_from_resource(&store, &mut save_state, &mut recovery_banner);
+            }
+            SaveLoadRequest::SaveManual { label, slot_id } => {
+                let snapshot = match_session.to_snapshot(SnapshotMetadata {
+                    label: label.clone(),
+                    created_at_utc: None,
+                    updated_at_utc: None,
+                    notes: Some(String::from("Manual save")),
+                    save_kind: SaveKind::Manual,
+                    session_id: slot_id.clone().unwrap_or_default(),
+                    recovery_key: None,
+                });
+
+                match store.0.save_manual(snapshot) {
+                    Ok(summary) => {
+                        save_state.last_error = None;
+                        save_state.last_message = Some(format!("Saved match as {}.", summary.label));
+                        menu_state.selected_save = Some(summary.slot_id.clone());
+                        refresh_store_index_from_resource(&store, &mut save_state, &mut recovery_banner);
+                    }
+                    Err(_) => {
+                        save_state.last_error = Some(String::from("Unable to write the selected save slot."));
+                    }
+                }
+            }
+            SaveLoadRequest::LoadManual { slot_id } => match store.0.load_manual(slot_id) {
+                Ok(snapshot) => {
+                    pending_snapshot.0 = Some(snapshot);
+                    *launch_intent = MatchLaunchIntent::LoadManual;
+                    save_state.last_error = None;
+                    save_state.last_message = Some(format!("Loading save {slot_id}."));
+                    next_state.set(AppScreenState::MatchLoading);
+                }
+                Err(_) => {
+                    save_state.last_error = Some(String::from("Unable to load the selected save."));
+                }
+            },
+            SaveLoadRequest::DeleteManual { slot_id } => match store.0.delete_manual(slot_id) {
+                Ok(()) => {
+                    save_state.last_error = None;
+                    save_state.last_message = Some(format!("Deleted save {slot_id}."));
+                    if menu_state.selected_save.as_deref() == Some(slot_id) {
+                        menu_state.selected_save = None;
+                    }
+                    refresh_store_index_from_resource(&store, &mut save_state, &mut recovery_banner);
+                }
+                Err(_) => {
+                    save_state.last_error = Some(String::from("Unable to delete the selected save."));
+                }
+            },
+            SaveLoadRequest::ResumeRecovery => match store.0.load_recovery() {
+                Ok(Some(snapshot)) => {
+                    pending_snapshot.0 = Some(snapshot);
+                    *launch_intent = MatchLaunchIntent::ResumeRecovery;
+                    save_state.last_error = None;
+                    save_state.last_message = Some(String::from("Resuming interrupted session."));
+                    next_state.set(AppScreenState::MatchLoading);
+                }
+                Ok(None) => {
+                    save_state.last_error = Some(String::from("No interrupted session is available."));
+                }
+                Err(_) => {
+                    save_state.last_error = Some(String::from("Unable to resume the interrupted session."));
+                }
+            },
+            SaveLoadRequest::ClearRecovery => match store.0.clear_recovery() {
+                Ok(()) => {
+                    recovery_banner.available = false;
+                    recovery_banner.dirty = false;
+                    save_state.recovery = None;
+                }
+                Err(_) => {
+                    save_state.last_error = Some(String::from("Unable to clear interrupted-session recovery."));
+                }
+            },
+            SaveLoadRequest::PersistSettings => match store.0.save_settings(&save_state.settings) {
+                Ok(()) => {
+                    save_state.last_error = None;
+                    save_state.last_message = Some(String::from("Saved shell settings."));
+                }
+                Err(_) => {
+                    save_state.last_error = Some(String::from("Unable to save shell settings."));
+                }
+            },
+        }
+    }
+}
+
+fn autosave_active_match(
+    state: Res<State<AppScreenState>>,
+    store: Res<SessionStoreResource>,
+    mut match_session: ResMut<MatchSession>,
+    mut save_state: ResMut<SaveLoadState>,
+    mut recovery_banner: ResMut<RecoveryBannerState>,
+) {
+    if !matches!(state.get(), AppScreenState::InMatch | AppScreenState::Paused)
+        || !match_session.is_changed()
+    {
+        return;
+    }
+
+    let snapshot = match_session.to_snapshot(SnapshotMetadata {
+        label: String::from("Interrupted Session"),
+        created_at_utc: None,
+        updated_at_utc: None,
+        notes: Some(String::from("Automatic recovery snapshot")),
+        save_kind: SaveKind::Recovery,
+        session_id: String::new(),
+        recovery_key: Some(String::from("autosave")),
+    });
+
+    match store.0.store_recovery(snapshot) {
+        Ok(summary) => {
+            match_session.mark_recovery_persisted();
+            save_state.recovery = Some(summary.clone());
+            recovery_banner.available = true;
+            recovery_banner.dirty = match_session.is_recovery_dirty();
+            recovery_banner.label = Some(summary.label);
+        }
+        Err(_) => {
+            save_state.last_error = Some(String::from("Unable to refresh interrupted-session recovery."));
+        }
+    }
+}
+
+fn clear_result_recovery(
+    store: Res<SessionStoreResource>,
+    mut save_state: ResMut<SaveLoadState>,
+    mut recovery_banner: ResMut<RecoveryBannerState>,
+) {
+    let _ = store.0.clear_recovery();
+    save_state.recovery = None;
+    recovery_banner.available = false;
+    recovery_banner.dirty = false;
+}
+
+fn refresh_store_index(
+    commands: &Commands,
+    save_state: &mut SaveLoadState,
+    recovery_banner: &mut RecoveryBannerState,
+) {
+    let Some(store) = commands.get_resource::<SessionStoreResource>() else {
+        return;
+    };
+    refresh_store_index_from_resource(store, save_state, recovery_banner);
+}
+
+fn refresh_store_index_from_resource(
+    store: &SessionStoreResource,
+    save_state: &mut SaveLoadState,
+    recovery_banner: &mut RecoveryBannerState,
+) {
+    save_state.manual_saves = store.0.list_manual_saves().unwrap_or_default();
+    save_state.settings = store.0.load_settings().unwrap_or_default();
+    save_state.recovery = store
+        .0
+        .load_recovery()
+        .ok()
+        .flatten()
+        .map(|snapshot| SavedSessionSummary::from_snapshot(&snapshot));
+    recovery_banner.available = save_state.recovery.is_some();
+    recovery_banner.label = save_state.recovery.as_ref().map(|summary| summary.label.clone());
+    if save_state.recovery.is_none() {
+        recovery_banner.dirty = false;
+    }
+}

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -1,4 +1,7 @@
+//! Shell persistence orchestration for manual saves, interrupted-session recovery, and settings.
+//! Repository I/O lives here so manual saves, interrupted-session recovery, and the shipped settings trio of startup recovery, destructive confirmations, and display mode stay behind one snapshot-based boundary. (ref: DL-002) (ref: DL-005) (ref: DL-007) (ref: DL-008)
+
 use std::path::PathBuf;
 
 use bevy::prelude::*;
@@ -10,6 +13,8 @@ use super::menu::{MenuPanel, RecoveryBannerState, ShellMenuState};
 use crate::app::AppScreenState;
 use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
 
+/// Wires session-store setup plus startup recovery, destructive confirmation, and display-mode persistence into the coarse app shell. (ref: DL-003) (ref: DL-005)
 pub struct SaveLoadPlugin;
@@ -52,6 +57,8 @@ impl Plugin for SaveLoadPlugin {
     }
 }
 
+/// Resolves the session-store root once so runtime builds use platform app-data and tests can inject deterministic temp roots. (ref: DL-008)
 fn setup_store(
     mut commands: Commands,
     root_override: Res<SaveRootOverride>,
@@ -76,6 +83,9 @@ fn setup_store(
     refresh_store_index(&commands, &mut save_state, &mut recovery_banner);
 }
 
+/// Applies the startup recovery policy before the user enters a match so resume behavior is predictable and separate from manual saves. (ref: DL-003) (ref: DL-005)
 fn maybe_resume_recovery_on_startup(
     state: Res<State<AppScreenState>>,
     mut handled: ResMut<StartupRecoveryHandled>,
@@ -131,6 +141,8 @@ fn apply_display_mode_setting(
     };
 }
 
+/// Executes shell save/load requests while keeping `MatchSession` responsible only for snapshot conversion. (ref: DL-002) (ref: DL-007)
 fn handle_save_load_requests(
     mut requests: EventReader<SaveLoadRequest>,
     store: Res<SessionStoreResource>,
@@ -225,6 +237,8 @@ fn handle_save_load_requests(
     }
 }
 
+/// Refreshes the single recovery record whenever the active match changes so interrupted-session resume reads a repository-backed snapshot of that match. (ref: DL-003)
 fn autosave_active_match(
     state: Res<State<AppScreenState>>,
     store: Res<SessionStoreResource>,

```


**CC-M-002-007** (crates/game_app/src/lib.rs) - implements CI-M-002-010

**Code:**

```diff
--- a/crates/game_app/src/lib.rs
+++ b/crates/game_app/src/lib.rs
@@ -6,9 +6,14 @@
 mod style;
 
 pub use app::{APP_TITLE, AppScreenState, build_app, run};
-pub use match_state::{ClaimedDrawReason, MatchSession};
+pub use match_state::{
+    ClaimedDrawReason, MatchLaunchIntent, MatchSession, MatchSessionSummary,
+    PendingLoadedSnapshot,
+};
 pub use plugins::{
-    AppShellPlugin, BoardScenePlugin, BoardSquareVisual, MoveFeedbackPlugin, PieceViewPlugin,
-    PieceVisual, ShellInputPlugin,
+    AppShellPlugin, BoardScenePlugin, BoardSquareVisual, ConfirmationKind, MenuAction,
+    MenuPanel, MenuPlugin, MoveFeedbackPlugin, PieceViewPlugin, PieceVisual,
+    RecoveryBannerState, SaveLoadPlugin, SaveLoadRequest, SaveLoadState,
+    SaveRootOverride, SessionStoreResource, ShellInputPlugin, ShellMenuState,
 };
 pub use style::ShellTheme;

```

**Documentation:**

```diff
--- a/crates/game_app/src/lib.rs
+++ b/crates/game_app/src/lib.rs
@@ -1,4 +1,7 @@
+//! Public game shell surface for local play, menu flow, and persistence integration.
+//! The exported types keep tests and top-level app wiring aligned on the same routing and repository boundaries. (ref: DL-001) (ref: DL-007)
+
 mod app;
 mod board_coords;
 mod match_state;

```


**CC-M-002-008** (crates/game_app/Cargo.toml) - implements CI-M-002-011

**Code:**

```diff
--- a/crates/game_app/Cargo.toml
+++ b/crates/game_app/Cargo.toml
@@ -18,5 +18,8 @@
 chess_persistence = { path = "../chess_persistence" }
 engine_uci = { path = "../engine_uci" }
 
+[dev-dependencies]
+tempfile = "3.15.0"
+
 [lints]
 workspace = true

```

**Documentation:**

```diff
--- a/crates/game_app/Cargo.toml
+++ b/crates/game_app/Cargo.toml
@@ -18,6 +18,7 @@
 engine_uci = { path = "../engine_uci" }
 
 [dev-dependencies]
+# Temp-rooted integration tests exercise platform-path behavior without changing the runtime default storage root. (ref: DL-008)
 tempfile = "3.15.0"
 
 [lints]

```


**CC-M-002-009** (crates/game_app/src/plugins/save_load.rs) - implements CI-M-002-006

**Code:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -7,6 +7,6 @@     ShellSettings, SnapshotMetadata,
 };
 
-use super::menu::{MenuPanel, RecoveryBannerState, ShellMenuState};
+use super::menu::{RecoveryBannerState, ShellMenuState};
 use crate::app::AppScreenState;
 use crate::match_state::{MatchLaunchIntent, MatchSession, PendingLoadedSnapshot};
 
@@ -76,7 +76,20 @@         .or_else(|| SessionStore::runtime().ok())
         .unwrap_or_else(|| SessionStore::new(std::env::temp_dir().join("3d-chess")));
 
+    save_state.manual_saves = store.list_manual_saves().unwrap_or_default();
+    save_state.settings = store.load_settings().unwrap_or_default();
+    save_state.recovery = store
+        .load_recovery()
+        .ok()
+        .flatten()
+        .map(|snapshot| SavedSessionSummary::from_snapshot(&snapshot));
+    recovery_banner.available = save_state.recovery.is_some();
+    recovery_banner.label = save_state
+        .recovery
+        .as_ref()
+        .map(|summary| summary.label.clone());
+    recovery_banner.dirty = false;
+
     commands.insert_resource(SessionStoreResource(store));
-    refresh_store_index(&commands, &mut save_state, &mut recovery_banner);
 }
 
 fn maybe_resume_recovery_on_startup(
@@ -237,12 +250,10 @@     mut save_state: ResMut<SaveLoadState>,
     mut recovery_banner: ResMut<RecoveryBannerState>,
 ) {
-    if !matches!(state.get(), AppScreenState::InMatch | AppScreenState::Paused)
-        || !match_session.is_changed()
-    {
-        return;
-    }
-
-    let snapshot = match_session.to_snapshot(SnapshotMetadata {
+    if *state.get() != AppScreenState::InMatch || !match_session.is_changed() {
+        return;
+    }
+
+    let mut snapshot = match_session.to_snapshot(SnapshotMetadata {
         label: String::from("Interrupted Session"),
         created_at_utc: None,
         updated_at_utc: None,
@@ -252,12 +263,13 @@         session_id: String::new(),
         recovery_key: Some(String::from("autosave")),
     });
+    snapshot.shell_state.dirty_recovery = false;
 
     match store.0.store_recovery(snapshot) {
         Ok(summary) => {
             match_session.mark_recovery_persisted();
             save_state.recovery = Some(summary.clone());
             recovery_banner.available = true;
-            recovery_banner.dirty = match_session.is_recovery_dirty();
+            recovery_banner.dirty = false;
             recovery_banner.label = Some(summary.label);
         }
         Err(_) => {
@@ -304,6 +316,4 @@         .map(|snapshot| SavedSessionSummary::from_snapshot(&snapshot));
     recovery_banner.available = save_state.recovery.is_some();
     recovery_banner.label = save_state.recovery.as_ref().map(|summary| summary.label.clone());
-    if save_state.recovery.is_none() {
-        recovery_banner.dirty = false;
-    }
-}
+    recovery_banner.dirty = false;
+}
```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/save_load.rs
+++ b/crates/game_app/src/plugins/save_load.rs
@@ -76,6 +76,7 @@ fn setup_store(
         .or_else(|| SessionStore::runtime().ok())
         .unwrap_or_else(|| SessionStore::new(std::env::temp_dir().join("3d-chess")));
 
+    // Startup preloads the save index and recovery banner from the repository so the main menu reflects persisted shell state immediately. (ref: DL-003) (ref: DL-008)
     save_state.manual_saves = store.list_manual_saves().unwrap_or_default();
     save_state.settings = store.load_settings().unwrap_or_default();
     save_state.recovery = store
@@ -262,6 +263,7 @@ fn autosave_active_match(
         session_id: String::new(),
         recovery_key: Some(String::from("autosave")),
     });
+    // The on-disk recovery record represents the last persisted state, so the dirty flag clears before write. (ref: DL-003)
     snapshot.shell_state.dirty_recovery = false;
 
     match store.0.store_recovery(snapshot) {
@@ -316,6 +318,7 @@ fn refresh_store_index_from_resource(
     save_state.recovery = store
         .0
         .load_recovery()
+        // Banner state recomputes from storage here so stale dirty flags never leak back into the shell. (ref: DL-003)
         .ok()
         .flatten()
         .map(|snapshot| SavedSessionSummary::from_snapshot(&snapshot));

```


**CC-M-002-010** (crates/game_app/src/plugins/app_shell.rs) - implements CI-M-002-007

**Code:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@ -1,8 +1,15 @@
 use bevy::prelude::*;
 use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, PieceKind, WinReason};
+use chess_persistence::{DisplayMode, RecoveryStartupPolicy, SavedSessionSummary};
 
+use super::menu::{
+    ConfirmationKind, MenuAction, MenuContext, MenuPanel, RecoveryBannerState, ShellMenuState,
+};
+use super::save_load::{SaveLoadRequest, SaveLoadState};
 use crate::app::AppScreenState;
-use crate::match_state::{ClaimedDrawReason, MatchSession};
+use crate::match_state::{
+    ClaimedDrawReason, MatchLaunchIntent, MatchSession, PendingLoadedSnapshot,
+};
 use crate::style::ShellTheme;
 
 pub struct AppShellPlugin;
@@ -11,16 +18,21 @@ impl Plugin for AppShellPlugin {
     fn build(&self, app: &mut App) {
         app.add_systems(Startup, (configure_ambient_light, spawn_shell_camera))
             .add_systems(OnEnter(AppScreenState::Boot), advance_to_main_menu)
-            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_shell_ui)
-            .add_systems(OnExit(AppScreenState::MainMenu), cleanup_main_menu_ui)
-            .add_systems(OnEnter(AppScreenState::MatchLoading), initialize_local_match)
+            .add_systems(OnEnter(AppScreenState::MainMenu), spawn_main_menu_ui)
+            .add_systems(
+                OnEnter(AppScreenState::MatchLoading),
+                resolve_match_launch_intent,
+            )
             .add_systems(OnEnter(AppScreenState::MatchResult), spawn_match_result_ui)
+            .add_systems(OnExit(AppScreenState::MainMenu), cleanup_shell_overlay)
+            .add_systems(OnExit(AppScreenState::InMatch), cleanup_shell_overlay)
             .add_systems(OnExit(AppScreenState::InMatch), cleanup_promotion_overlay)
             .add_systems(OnExit(AppScreenState::MatchResult), cleanup_match_result_ui)
             .add_systems(
                 Update,
                 (
-                    orbit_camera.run_if(in_state(AppScreenState::MainMenu)),
+                    orbit_camera,
+                    refresh_shell_overlay,
                     sync_promotion_overlay.run_if(in_state(AppScreenState::InMatch)),
                     handle_shell_button_actions,
                     advance_to_match_result.run_if(in_state(AppScreenState::InMatch)),
@@ -30,7 +42,7 @@ impl Plugin for AppShellPlugin {
 }
 
 #[derive(Component)]
-struct MainMenuUi;
+struct ShellOverlayUi;
 
 #[derive(Component)]
 struct MatchResultUi;
@@ -38,16 +50,32 @@ struct MatchResultUi;
 #[derive(Component)]
 struct PromotionOverlayUi;
 
-#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
+#[derive(Component, Debug, Clone, PartialEq, Eq)]
 struct ShellActionButton {
     action: ShellAction,
 }
 
-#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+#[derive(Debug, Clone, PartialEq, Eq)]
 enum ShellAction {
-    StartLocalMatch,
-    Rematch,
+    OpenSetup,
+    BackToSetup,
+    StartNewMatch,
+    OpenLoadList,
+    OpenSettings,
+    ResumeRecovery,
+    ResumeMatch,
     ReturnToMenu,
+    Rematch,
+    SaveManual,
+    OverwriteSelectedSave,
+    LoadSelected,
+    DeleteSelected,
+    SelectSave(String),
+    CycleRecoveryPolicy,
+    ToggleDisplayMode,
+    ToggleConfirmation(ConfirmationKind),
+    CancelModal,
+    Confirm(ConfirmationKind),
     Promote(PieceKind),
 }
 
@@ -101,7 +129,28 @@ fn advance_to_main_menu(mut next_state: ResMut<NextState<AppScreenState>>) {
     next_state.set(AppScreenState::MainMenu);
 }
 
-fn spawn_shell_ui(mut commands: Commands, theme: Res<ShellTheme>) {
+fn spawn_main_menu_ui(
+    mut commands: Commands,
+    theme: Res<ShellTheme>,
+    menu_state: Res<ShellMenuState>,
+    save_state: Res<SaveLoadState>,
+    recovery: Res<RecoveryBannerState>,
+) {
+    if matches!(menu_state.panel, MenuPanel::Home) {
+        build_main_menu_ui(&mut commands, theme.as_ref(), recovery.as_ref());
+    } else {
+        build_setup_ui(
+            &mut commands,
+            theme.as_ref(),
+            menu_state.as_ref(),
+            save_state.as_ref(),
+            recovery.as_ref(),
+            false,
+        );
+    }
+}
+
+fn build_main_menu_ui(commands: &mut Commands, theme: &ShellTheme, recovery: &RecoveryBannerState) {
     commands
         .spawn((
             Node {
@@ -113,13 +162,13 @@ fn spawn_shell_ui(mut commands: Commands, theme: Res<ShellTheme>) {
                 padding: UiRect::axes(Val::Px(24.0), Val::Px(24.0)),
                 ..default()
             },
-            MainMenuUi,
+            ShellOverlayUi,
         ))
         .with_children(|parent| {
             parent
                 .spawn((
                     Node {
-                        width: Val::Px(420.0),
+                        width: Val::Px(460.0),
                         flex_direction: FlexDirection::Column,
                         row_gap: Val::Px(10.0),
                         padding: UiRect::all(Val::Px(18.0)),
@@ -137,7 +186,7 @@ fn spawn_shell_ui(mut commands: Commands, theme: Res<ShellTheme>) {
                         TextColor(theme.ui_text),
                     ));
                     panel.spawn((
-                        Text::new("Start a local 3D match"),
+                        Text::new("M3 completes the local product shell"),
                         TextFont {
                             font_size: 20.0,
                             ..default()
@@ -146,8 +195,7 @@ fn spawn_shell_ui(mut commands: Commands, theme: Res<ShellTheme>) {
                     ));
                     panel.spawn((
                         Text::new(
-                            "M2 begins the playable shell: start a local match now, keep chess_core \
-authoritative, and leave wider shell work for later milestones.",
+                            "Open local match setup, manage saves, and resume interrupted sessions without widening top-level routing.",
                         ),
                         TextFont {
                             font_size: 16.0,
@@ -155,58 +203,429 @@ authoritative, and leave wider shell work for later milestones.",
                         },
                         TextColor(theme.ui_text),
                     ));
-
                     spawn_action_button(
                         panel,
-                        "Start Local Match",
-                        theme.as_ref(),
-                        ShellAction::StartLocalMatch,
+                        "Local Match Setup",
+                        theme,
+                        ShellAction::OpenSetup,
                         true,
                     );
+                    if recovery.available {
+                        spawn_action_button(
+                            panel,
+                            "Resume Interrupted Match",
+                            theme,
+                            ShellAction::ResumeRecovery,
+                            false,
+                        );
+                    }
                 });
+        });
+}
 
+fn build_setup_ui(
+    commands: &mut Commands,
+    theme: &ShellTheme,
+    menu_state: &ShellMenuState,
+    save_state: &SaveLoadState,
+    recovery: &RecoveryBannerState,
+    paused: bool,
+) {
+    let title = if paused {
+        "Paused"
+    } else {
+        "Local Match Setup"
+    };
+    let subtitle = if paused {
+        "Save, load, or abandon without bypassing recovery safeguards."
+    } else {
+        "Choose how the next local session should begin."
+    };
+
+    commands
+        .spawn((
+            Node {
+                width: Val::Percent(100.0),
+                height: Val::Percent(100.0),
+                justify_content: JustifyContent::Center,
+                align_items: AlignItems::Center,
+                padding: UiRect::all(Val::Px(24.0)),
+                ..default()
+            },
+            ShellOverlayUi,
+        ))
+        .with_children(|parent| {
             parent
                 .spawn((
                     Node {
-                        align_self: AlignSelf::FlexEnd,
-                        width: Val::Px(360.0),
+                        width: Val::Px(560.0),
                         flex_direction: FlexDirection::Column,
-                        row_gap: Val::Px(6.0),
-                        padding: UiRect::all(Val::Px(16.0)),
+                        row_gap: Val::Px(14.0),
+                        padding: UiRect::all(Val::Px(22.0)),
                         ..default()
                     },
-                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.60)),
+                    BackgroundColor(theme.ui_panel),
                 ))
                 .with_children(|panel| {
-                    for line in [
-                        "Match session lives in game_app and wraps chess_core",
-                        "Result transitions observe domain status only",
-                        "Stockfish/UCI boundary remains reserved for M4",
-                    ] {
+                    panel.spawn((
+                        Text::new(title),
+                        TextFont {
+                            font_size: 34.0,
+                            ..default()
+                        },
+                        TextColor(theme.ui_text),
+                    ));
+                    panel.spawn((
+                        Text::new(subtitle),
+                        TextFont {
+                            font_size: 18.0,
+                            ..default()
+                        },
+                        TextColor(theme.accent),
+                    ));
+
+                    if let Some(status) = effective_shell_status(menu_state, save_state, recovery) {
                         panel.spawn((
-                            Text::new(line),
+                            Text::new(status),
                             TextFont {
-                                font_size: 15.0,
+                                font_size: 14.0,
                                 ..default()
                             },
                             TextColor(theme.ui_text),
                         ));
                     }
+
+                    match menu_state.panel {
+                        MenuPanel::Home | MenuPanel::Setup => {
+                            if paused {
+                                spawn_action_button(
+                                    panel,
+                                    "Resume Match",
+                                    theme,
+                                    ShellAction::ResumeMatch,
+                                    true,
+                                );
+                                spawn_action_button(
+                                    panel,
+                                    "Create Manual Save",
+                                    theme,
+                                    ShellAction::SaveManual,
+                                    false,
+                                );
+                            } else {
+                                spawn_action_button(
+                                    panel,
+                                    "Start New Match",
+                                    theme,
+                                    ShellAction::StartNewMatch,
+                                    true,
+                                );
+                            }
+
+                            spawn_action_button(
+                                panel,
+                                "Open Save Slots",
+                                theme,
+                                ShellAction::OpenLoadList,
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                "Settings",
+                                theme,
+                                ShellAction::OpenSettings,
+                                false,
+                            );
+
+                            if recovery.available {
+                                spawn_action_button(
+                                    panel,
+                                    "Resume Interrupted Match",
+                                    theme,
+                                    ShellAction::ResumeRecovery,
+                                    false,
+                                );
+                            }
+
+                            spawn_action_button(
+                                panel,
+                                if paused {
+                                    "Return to Main Menu"
+                                } else {
+                                    "Back to Main Menu"
+                                },
+                                theme,
+                                ShellAction::ReturnToMenu,
+                                false,
+                            );
+                        }
+                        MenuPanel::LoadList => {
+                            if save_state.manual_saves.is_empty() {
+                                panel.spawn((
+                                    Text::new("No manual saves are available yet."),
+                                    TextFont {
+                                        font_size: 15.0,
+                                        ..default()
+                                    },
+                                    TextColor(theme.ui_text),
+                                ));
+                            } else {
+                                for save in &save_state.manual_saves {
+                                    let label = if menu_state.selected_save.as_deref()
+                                        == Some(save.slot_id.as_str())
+                                    {
+                                        format!("> {}", save.label)
+                                    } else {
+                                        save.label.clone()
+                                    };
+                                    spawn_action_button(
+                                        panel,
+                                        &label,
+                                        theme,
+                                        ShellAction::SelectSave(save.slot_id.clone()),
+                                        menu_state.selected_save.as_deref()
+                                            == Some(save.slot_id.as_str()),
+                                    );
+                                }
+                            }
+
+                            spawn_action_button(
+                                panel,
+                                "Load Selected Save",
+                                theme,
+                                ShellAction::LoadSelected,
+                                true,
+                            );
+                            if paused {
+                                spawn_action_button(
+                                    panel,
+                                    "Overwrite Selected Save",
+                                    theme,
+                                    ShellAction::OverwriteSelectedSave,
+                                    false,
+                                );
+                            }
+                            spawn_action_button(
+                                panel,
+                                "Delete Selected Save",
+                                theme,
+                                ShellAction::DeleteSelected,
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                "Back",
+                                theme,
+                                ShellAction::BackToSetup,
+                                false,
+                            );
+                        }
+                        MenuPanel::Settings => {
+                            panel.spawn((
+                                Text::new(format!(
+                                    "Startup recovery: {}",
+                                    recovery_policy_label(save_state.settings.recovery_policy)
+                                )),
+                                TextFont {
+                                    font_size: 15.0,
+                                    ..default()
+                                },
+                                TextColor(theme.ui_text),
+                            ));
+                            spawn_action_button(
+                                panel,
+                                "Cycle Startup Recovery",
+                                theme,
+                                ShellAction::CycleRecoveryPolicy,
+                                false,
+                            );
+                            panel.spawn((
+                                Text::new(format!(
+                                    "Display mode: {}",
+                                    display_mode_label(save_state.settings.display_mode)
+                                )),
+                                TextFont {
+                                    font_size: 15.0,
+                                    ..default()
+                                },
+                                TextColor(theme.ui_text),
+                            ));
+                            spawn_action_button(
+                                panel,
+                                "Toggle Display Mode",
+                                theme,
+                                ShellAction::ToggleDisplayMode,
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                &toggle_label(
+                                    "Confirm menu abandon",
+                                    save_state.settings.confirm_actions.abandon_match,
+                                ),
+                                theme,
+                                ShellAction::ToggleConfirmation(ConfirmationKind::AbandonMatch),
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                &toggle_label(
+                                    "Confirm save delete",
+                                    save_state.settings.confirm_actions.delete_save,
+                                ),
+                                theme,
+                                ShellAction::ToggleConfirmation(ConfirmationKind::DeleteSave),
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                &toggle_label(
+                                    "Confirm save overwrite",
+                                    save_state.settings.confirm_actions.overwrite_save,
+                                ),
+                                theme,
+                                ShellAction::ToggleConfirmation(ConfirmationKind::OverwriteSave),
+                                false,
+                            );
+                            spawn_action_button(
+                                panel,
+                                "Back",
+                                theme,
+                                ShellAction::BackToSetup,
+                                false,
+                            );
+                        }
+                    }
+
+                    if let Some(kind) = menu_state.confirmation {
+                        let (headline, detail) = confirmation_copy(kind);
+                        panel.spawn((
+                            Text::new(headline),
+                            TextFont {
+                                font_size: 18.0,
+                                ..default()
+                            },
+                            TextColor(theme.accent),
+                        ));
+                        panel.spawn((
+                            Text::new(detail),
+                            TextFont {
+                                font_size: 14.0,
+                                ..default()
+                            },
+                            TextColor(theme.ui_text),
+                        ));
+                        spawn_action_button(
+                            panel,
+                            "Confirm",
+                            theme,
+                            ShellAction::Confirm(kind),
+                            true,
+                        );
+                        spawn_action_button(
+                            panel,
+                            "Cancel",
+                            theme,
+                            ShellAction::CancelModal,
+                            false,
+                        );
+                    }
                 });
         });
 }
 
-fn cleanup_main_menu_ui(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUi>>) {
-    for entity in &menu_query {
+fn refresh_shell_overlay(
+    state: Res<State<AppScreenState>>,
+    theme: Res<ShellTheme>,
+    menu_state: Res<ShellMenuState>,
+    save_state: Res<SaveLoadState>,
+    recovery: Res<RecoveryBannerState>,
+    overlay_query: Query<Entity, With<ShellOverlayUi>>,
+    mut commands: Commands,
+) {
+    let render_main_menu = *state.get() == AppScreenState::MainMenu;
+    let render_pause_overlay = *state.get() == AppScreenState::InMatch
+        && menu_state.context == MenuContext::InMatchOverlay;
+
+    if !render_main_menu && !render_pause_overlay {
+        for entity in &overlay_query {
+            commands.entity(entity).despawn();
+        }
+        return;
+    }
+
+    if !(menu_state.is_changed() || save_state.is_changed() || recovery.is_changed())
+        && !overlay_query.is_empty()
+    {
+        return;
+    }
+
+    for entity in &overlay_query {
+        commands.entity(entity).despawn();
+    }
+
+    if render_main_menu {
+        if matches!(menu_state.panel, MenuPanel::Home) {
+            build_main_menu_ui(&mut commands, theme.as_ref(), recovery.as_ref());
+        } else {
+            build_setup_ui(
+                &mut commands,
+                theme.as_ref(),
+                menu_state.as_ref(),
+                save_state.as_ref(),
+                recovery.as_ref(),
+                false,
+            );
+        }
+        return;
+    }
+
+    build_setup_ui(
+        &mut commands,
+        theme.as_ref(),
+        menu_state.as_ref(),
+        save_state.as_ref(),
+        recovery.as_ref(),
+        true,
+    );
+}
+
+fn cleanup_shell_overlay(
+    mut commands: Commands,
+    overlay_query: Query<Entity, With<ShellOverlayUi>>,
+) {
+    for entity in &overlay_query {
         commands.entity(entity).despawn();
     }
 }
 
-fn initialize_local_match(
+fn resolve_match_launch_intent(
     mut match_session: ResMut<MatchSession>,
+    mut launch_intent: ResMut<MatchLaunchIntent>,
+    mut pending_snapshot: ResMut<PendingLoadedSnapshot>,
+    mut menu_state: ResMut<ShellMenuState>,
     mut next_state: ResMut<NextState<AppScreenState>>,
 ) {
-    match_session.reset_for_local_match();
+    match *launch_intent {
+        MatchLaunchIntent::NewLocalMatch | MatchLaunchIntent::Rematch => {
+            match_session.reset_for_local_match();
+        }
+        MatchLaunchIntent::LoadManual | MatchLaunchIntent::ResumeRecovery => {
+            let Some(snapshot) = pending_snapshot.0.take() else {
+                menu_state.status_line = Some(String::from("No saved session was ready to load."));
+                menu_state.context = MenuContext::MainMenu;
+                menu_state.panel = MenuPanel::Setup;
+                next_state.set(AppScreenState::MainMenu);
+                return;
+            };
+            *match_session = MatchSession::restore_from_snapshot(&snapshot);
+        }
+    }
+
+    *launch_intent = MatchLaunchIntent::NewLocalMatch;
+    menu_state.context = MenuContext::MainMenu;
+    menu_state.panel = MenuPanel::Setup;
+    menu_state.confirmation = None;
     next_state.set(AppScreenState::InMatch);
 }
 
@@ -259,18 +678,6 @@ fn spawn_match_result_ui(
                         },
                         TextColor(theme.accent),
                     ));
-                    panel.spawn((
-                        Text::new(
-                            "Rematch resets the domain session to the starting position. \
-Return to Menu keeps the shell path narrow until broader M3 flows land.",
-                        ),
-                        TextFont {
-                            font_size: 15.0,
-                            ..default()
-                        },
-                        TextColor(theme.ui_text),
-                    ));
-
                     spawn_action_button(
                         panel,
                         "Rematch",
@@ -280,7 +687,7 @@ Return to Menu keeps the shell path narrow until broader M3 flows land.",
                     );
                     spawn_action_button(
                         panel,
-                        "Return to Menu",
+                        "Return to Main Menu",
                         theme.as_ref(),
                         ShellAction::ReturnToMenu,
                         false,
@@ -389,27 +796,143 @@ fn cleanup_promotion_overlay(
 
 fn handle_shell_button_actions(
     interaction_query: Query<(&Interaction, &ShellActionButton), Changed<Interaction>>,
-    mut match_session: ResMut<MatchSession>,
-    mut next_state: ResMut<NextState<AppScreenState>>,
+    state: Res<State<AppScreenState>>,
+    menu_state: Res<ShellMenuState>,
+    mut save_state: ResMut<SaveLoadState>,
+    mut menu_actions: EventWriter<MenuAction>,
+    mut save_requests: EventWriter<SaveLoadRequest>,
+    mut match_session_mut: ResMut<MatchSession>,
 ) {
     for (interaction, button_action) in &interaction_query {
         if *interaction != Interaction::Pressed {
             continue;
         }
 
-        match button_action.action {
-            ShellAction::StartLocalMatch | ShellAction::Rematch => {
-                next_state.set(AppScreenState::MatchLoading);
+        match &button_action.action {
+            ShellAction::OpenSetup => menu_actions.send(MenuAction::OpenSetup),
+            ShellAction::BackToSetup => menu_actions.send(MenuAction::BackToSetup),
+            ShellAction::StartNewMatch => menu_actions.send(MenuAction::StartNewMatch),
+            ShellAction::OpenLoadList => menu_actions.send(MenuAction::OpenLoadList),
+            ShellAction::OpenSettings => menu_actions.send(MenuAction::OpenSettings),
+            ShellAction::ResumeRecovery => {
+                save_requests.send(SaveLoadRequest::ResumeRecovery);
             }
+            ShellAction::ResumeMatch => menu_actions.send(MenuAction::ResumeMatch),
             ShellAction::ReturnToMenu => {
-                next_state.set(AppScreenState::MainMenu);
+                if *state.get() == AppScreenState::InMatch
+                    && menu_state.context == MenuContext::InMatchOverlay
+                {
+                    if save_state.settings.confirm_actions.abandon_match {
+                        menu_actions.send(MenuAction::RequestConfirmation(
+                            ConfirmationKind::AbandonMatch,
+                        ));
+                    } else {
+                        save_requests.send(SaveLoadRequest::ClearRecovery);
+                        menu_actions.send(MenuAction::ReturnToMenu);
+                    }
+                } else {
+                    menu_actions.send(MenuAction::ReturnToMenu);
+                }
+            }
+            ShellAction::Rematch => menu_actions.send(MenuAction::Rematch),
+            ShellAction::SaveManual => {
+                save_requests.send(SaveLoadRequest::SaveManual {
+                    label: derive_save_label(match_session_mut.as_ref()),
+                    slot_id: None,
+                });
+            }
+            ShellAction::OverwriteSelectedSave => {
+                if let Some(selected) =
+                    selected_save_summary(menu_state.as_ref(), save_state.as_ref())
+                {
+                    if save_state.settings.confirm_actions.overwrite_save {
+                        menu_actions.send(MenuAction::RequestConfirmation(
+                            ConfirmationKind::OverwriteSave,
+                        ));
+                    } else {
+                        save_requests.send(SaveLoadRequest::SaveManual {
+                            label: selected.label.clone(),
+                            slot_id: Some(selected.slot_id.clone()),
+                        });
+                    }
+                }
+            }
+            ShellAction::LoadSelected => {
+                if let Some(slot_id) = menu_state.selected_save.clone() {
+                    save_requests.send(SaveLoadRequest::LoadManual { slot_id });
+                }
+            }
+            ShellAction::DeleteSelected => {
+                if let Some(slot_id) = menu_state.selected_save.clone() {
+                    if save_state.settings.confirm_actions.delete_save {
+                        menu_actions.send(MenuAction::RequestConfirmation(
+                            ConfirmationKind::DeleteSave,
+                        ));
+                    } else {
+                        save_requests.send(SaveLoadRequest::DeleteManual { slot_id });
+                    }
+                }
+            }
+            ShellAction::SelectSave(slot_id) => {
+                menu_actions.send(MenuAction::SelectSave(slot_id.clone()));
+            }
+            ShellAction::CycleRecoveryPolicy => {
+                save_state.settings.recovery_policy =
+                    next_recovery_policy(save_state.settings.recovery_policy);
+                save_requests.send(SaveLoadRequest::PersistSettings);
+            }
+            ShellAction::ToggleDisplayMode => {
+                save_state.settings.display_mode = match save_state.settings.display_mode {
+                    DisplayMode::Windowed => DisplayMode::Fullscreen,
+                    DisplayMode::Fullscreen => DisplayMode::Windowed,
+                };
+                save_requests.send(SaveLoadRequest::PersistSettings);
+            }
+            ShellAction::ToggleConfirmation(kind) => {
+                match kind {
+                    ConfirmationKind::AbandonMatch => {
+                        save_state.settings.confirm_actions.abandon_match =
+                            !save_state.settings.confirm_actions.abandon_match;
+                    }
+                    ConfirmationKind::DeleteSave => {
+                        save_state.settings.confirm_actions.delete_save =
+                            !save_state.settings.confirm_actions.delete_save;
+                    }
+                    ConfirmationKind::OverwriteSave => {
+                        save_state.settings.confirm_actions.overwrite_save =
+                            !save_state.settings.confirm_actions.overwrite_save;
+                    }
+                }
+                save_requests.send(SaveLoadRequest::PersistSettings);
+            }
+            ShellAction::CancelModal => menu_actions.send(MenuAction::CancelModal),
+            ShellAction::Confirm(kind) => {
+                match kind {
+                    ConfirmationKind::AbandonMatch => {
+                        save_requests.send(SaveLoadRequest::ClearRecovery);
+                        menu_actions.send(MenuAction::ReturnToMenu);
+                    }
+                    ConfirmationKind::DeleteSave => {
+                        if let Some(slot_id) = menu_state.selected_save.clone() {
+                            save_requests.send(SaveLoadRequest::DeleteManual { slot_id });
+                        }
+                    }
+                    ConfirmationKind::OverwriteSave => {
+                        if let Some(selected) =
+                            selected_save_summary(menu_state.as_ref(), save_state.as_ref())
+                        {
+                            save_requests.send(SaveLoadRequest::SaveManual {
+                                label: selected.label.clone(),
+                                slot_id: Some(selected.slot_id.clone()),
+                            });
+                        }
+                    }
+                }
+                menu_actions.send(MenuAction::CancelModal);
             }
             ShellAction::Promote(piece_kind) => {
-                if let Some(pending_move) = match_session.pending_promotion_move {
-                    let _ = match_session.apply_move(chess_core::Move::with_promotion(
+                if let Some(pending_move) = match_session_mut.pending_promotion_move {
+                    let _ = match_session_mut.apply_move(chess_core::Move::with_promotion(
                         pending_move.from(),
                         pending_move.to(),
-                        piece_kind,
+                        *piece_kind,
                     ));
                 }
             }
@@ -428,9 +951,14 @@ fn advance_to_match_result(
 
 fn orbit_camera(
     time: Res<Time>,
+    state: Res<State<AppScreenState>>,
     theme: Res<ShellTheme>,
     mut camera_query: Query<(&mut Transform, &mut ShellCamera)>,
 ) {
+    if *state.get() != AppScreenState::MainMenu {
+        return;
+    }
+
     for (mut transform, mut shell_camera) in &mut camera_query {
         shell_camera.orbit_angle += time.delta_secs() * theme.orbit_speed;
 
@@ -481,12 +1009,95 @@ fn spawn_action_button(
         });
 }
 
+fn effective_shell_status(
+    menu_state: &ShellMenuState,
+    save_state: &SaveLoadState,
+    recovery: &RecoveryBannerState,
+) -> Option<String> {
+    save_state
+        .last_error
+        .clone()
+        .or_else(|| save_state.last_message.clone())
+        .or_else(|| menu_state.status_line.clone())
+        .or_else(|| {
+            recovery
+                .label
+                .as_ref()
+                .map(|label| format!("Interrupted-session recovery is available as {label}."))
+        })
+}
+
+fn derive_save_label(match_session: &MatchSession) -> String {
+    if let Some(last_move) = match_session.last_move {
+        format!("Local Match after {last_move}")
+    } else {
+        String::from("Local Match Save")
+    }
+}
+
+fn selected_save_summary<'a>(
+    menu_state: &ShellMenuState,
+    save_state: &'a SaveLoadState,
+) -> Option<&'a SavedSessionSummary> {
+    let slot_id = menu_state.selected_save.as_deref()?;
+    save_state
+        .manual_saves
+        .iter()
+        .find(|summary| summary.slot_id == slot_id)
+}
+
+fn next_recovery_policy(current: RecoveryStartupPolicy) -> RecoveryStartupPolicy {
+    match current {
+        RecoveryStartupPolicy::Resume => RecoveryStartupPolicy::Ask,
+        RecoveryStartupPolicy::Ask => RecoveryStartupPolicy::Ignore,
+        RecoveryStartupPolicy::Ignore => RecoveryStartupPolicy::Resume,
+    }
+}
+
+fn recovery_policy_label(policy: RecoveryStartupPolicy) -> &'static str {
+    match policy {
+        RecoveryStartupPolicy::Resume => "Resume automatically",
+        RecoveryStartupPolicy::Ask => "Ask on startup",
+        RecoveryStartupPolicy::Ignore => "Ignore recovery on startup",
+    }
+}
+
+fn display_mode_label(mode: DisplayMode) -> &'static str {
+    match mode {
+        DisplayMode::Windowed => "Windowed",
+        DisplayMode::Fullscreen => "Fullscreen",
+    }
+}
+
+fn toggle_label(label: &str, enabled: bool) -> String {
+    if enabled {
+        format!("{label}: on")
+    } else {
+        format!("{label}: off")
+    }
+}
+
+fn confirmation_copy(kind: ConfirmationKind) -> (&'static str, &'static str) {
+    match kind {
+        ConfirmationKind::AbandonMatch => (
+            "Leave the current match?",
+            "Clearing the recovery slot prevents startup resume from restoring this position.",
+        ),
+        ConfirmationKind::DeleteSave => (
+            "Delete the selected save?",
+            "Manual save history is user-controlled so deletes stay explicit.",
+        ),
+        ConfirmationKind::OverwriteSave => (
+            "Overwrite the selected save?",
+            "Manual saves stay distinct from recovery, so overwrites should always be deliberate.",
+        ),
+    }
+}
+
 fn match_session_result_title(match_session: &MatchSession) -> String {
     if let Some(claimed_draw_reason) = match_session.claimed_draw_reason() {
         return match claimed_draw_reason {
-            ClaimedDrawReason::ThreefoldRepetition => {
-                String::from("Draw Claimed by Repetition")
-            }
+            ClaimedDrawReason::ThreefoldRepetition => String::from("Draw Claimed by Repetition"),
             ClaimedDrawReason::FiftyMoveRule => String::from("Draw Claimed by Fifty-Move Rule"),
         };
     }
@@ -517,9 +1128,9 @@ fn match_session_result_detail(match_session: &MatchSession) -> String {
     }
 
     match match_session.status() {
-        chess_core::GameStatus::Ongoing { .. } => String::from(
-            "The shell can now route into match results when chess_core reports a terminal state.",
-        ),
+        chess_core::GameStatus::Ongoing { .. } => {
+            String::from("The shell routes to results only after chess_core resolves the outcome.")
+        }
         chess_core::GameStatus::Finished(GameOutcome::Win {
             reason: WinReason::Checkmate,
             ..

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/app_shell.rs
+++ b/crates/game_app/src/plugins/app_shell.rs
@@ -1,6 +1,9 @@
+//! Presentation layer for the coarse app shell.
+//! Main menu, pause overlay, and results render from modal resources while match launch still funnels through MatchLoading. (ref: DL-001) (ref: DL-007)
+
 use bevy::prelude::*;
 use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, PieceKind, WinReason};
 use chess_persistence::{DisplayMode, RecoveryStartupPolicy, SavedSessionSummary};
@@ -185,6 +188,9 @@ fn build_main_menu_ui(commands: &mut Commands, theme: &ShellTheme, recovery: &Re
         });
 }
 
+/// Renders the setup/load/settings surface for both the main menu and the in-match pause overlay.
+/// The panel stays modal so setup, load, startup recovery, destructive confirmations, and display mode do not add more top-level app states. (ref: DL-001) (ref: DL-005)
 fn build_setup_ui(
     commands: &mut Commands,
     theme: &ShellTheme,
@@ -510,6 +516,9 @@ fn build_setup_ui(
         });
 }
 
+/// Rebuilds whichever shell overlay matches the coarse route and modal menu context.
+/// Rendering from resources keeps UI nodes disposable and leaves state ownership in dedicated shell resources. (ref: DL-001) (ref: DL-007)
 fn refresh_shell_overlay(
     state: Res<State<AppScreenState>>,
     theme: Res<ShellTheme>,
@@ -576,6 +585,9 @@ fn cleanup_shell_overlay(
     }
 }
 
+/// Consumes the explicit launch intent before entering `InMatch`.
+/// Match loading either resets the domain session or hydrates a pending snapshot, but it never guesses which path the user meant. (ref: DL-001)
 fn resolve_match_launch_intent(
     mut match_session: ResMut<MatchSession>,
     mut launch_intent: ResMut<MatchLaunchIntent>,
@@ -807,6 +819,8 @@ fn orbit_camera(
         });
 }
 
+/// Chooses the most actionable shell status line so save/load feedback and recovery availability share one predictable surface. (ref: DL-003)
 fn effective_shell_status(
     menu_state: &ShellMenuState,
     save_state: &SaveLoadState,
@@ -875,6 +889,8 @@ fn toggle_label(label: &str, enabled: bool) -> String {
     }
 }
 
+/// Supplies confirmation copy for the destructive-confirmation slice of the shipped shell settings contract. (ref: DL-005)
 fn confirmation_copy(kind: ConfirmationKind) -> (&'static str, &'static str) {
     match kind {
         ConfirmationKind::AbandonMatch => (

```


**CC-M-002-011** (crates/game_app/src/plugins/input.rs) - implements CI-M-002-008

**Code:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -2,6 +2,8 @@
 use bevy::window::PrimaryWindow;
 use chess_core::{Move, PieceKind};
 
+use super::menu::{MenuAction, ShellMenuState};
+use super::save_load::SaveLoadRequest;
 use crate::app::AppScreenState;
 use crate::board_coords::{board_plane_intersection, world_to_square};
 use crate::match_state::MatchSession;
@@ -10,7 +12,7 @@
 #[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
 struct HoveredSquare(Option<chess_core::Square>);
 
-// Input resolves to chess squares first and only then to domain actions so legal previews and move execution always flow through chess_core.
+// Input resolves to chess squares first and only then to shell events so recovery snapshots mirror domain intent.
 pub struct ShellInputPlugin;
 
 impl Plugin for ShellInputPlugin {
@@ -73,6 +75,7 @@
 
     let Some(clicked_square) = hovered_square.0 else {
         match_session.selected_square = None;
+        match_session.mark_recovery_dirty();
         return;
     };
     if match_session.pending_promotion_move.is_some() {
@@ -85,17 +88,20 @@
     let Some(selected_square) = match_session.selected_square else {
         if clicked_piece.is_some_and(|piece| piece.side == current_side) {
             match_session.selected_square = Some(clicked_square);
+            match_session.mark_recovery_dirty();
         }
         return;
     };
 
     if clicked_square == selected_square {
         match_session.clear_interaction();
+        match_session.mark_recovery_dirty();
         return;
     }
 
     if clicked_piece.is_some_and(|piece| piece.side == current_side) {
         match_session.selected_square = Some(clicked_square);
+        match_session.mark_recovery_dirty();
         return;
     }
 
@@ -108,11 +114,13 @@
 
     if candidate_moves.is_empty() {
         match_session.selected_square = None;
+        match_session.mark_recovery_dirty();
         return;
     }
 
     if candidate_moves.iter().any(|candidate| candidate.promotion().is_some()) {
         match_session.pending_promotion_move = Some(Move::new(selected_square, clicked_square));
+        match_session.mark_recovery_dirty();
         return;
     }
 
@@ -121,8 +129,10 @@
 
 fn handle_keyboard_match_actions(
     keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
+    menu_state: Res<ShellMenuState>,
     mut match_session: ResMut<MatchSession>,
-    mut next_state: ResMut<NextState<AppScreenState>>,
+    mut menu_actions: EventWriter<MenuAction>,
+    mut save_requests: EventWriter<SaveLoadRequest>,
 ) {
     let Some(keyboard_input) = keyboard_input else {
         return;
@@ -131,10 +141,20 @@
     if keyboard_input.just_pressed(KeyCode::Escape) {
         if match_session.pending_promotion_move.is_some() || match_session.selected_square.is_some() {
             match_session.clear_interaction();
+            match_session.mark_recovery_dirty();
+        } else if menu_state.confirmation.is_some() {
+            menu_actions.send(MenuAction::CancelModal);
         } else {
-            next_state.set(AppScreenState::MainMenu);
+            menu_actions.send(MenuAction::PauseMatch);
         }
         return;
+    }
+
+    if keyboard_input.just_pressed(KeyCode::F5) && match_session.pending_promotion_move.is_none() {
+        save_requests.send(SaveLoadRequest::SaveManual {
+            label: String::from("Quick Save"),
+            slot_id: None,
+        });
     }
 
     let Some(pending_move) = match_session.pending_promotion_move else {

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/input.rs
+++ b/crates/game_app/src/plugins/input.rs
@@ -129,6 +129,8 @@ fn handle_pointer_selection(
     let _ = match_session.apply_move(candidate_moves[0]);
 }
 
+/// Applies keyboard shell actions after promotion and selection handling so pause, cancel, and quick-save all preserve snapshot intent. (ref: DL-001) (ref: DL-003)
 fn handle_keyboard_match_actions(
     keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
     menu_state: Res<ShellMenuState>,

```


**CC-M-002-012** (crates/game_app/src/plugins/move_feedback.rs) - implements CI-M-002-009

**Code:**

```diff
--- a/crates/game_app/src/plugins/move_feedback.rs
+++ b/crates/game_app/src/plugins/move_feedback.rs
@@ -2,6 +2,7 @@
 use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Side, WinReason};
 
 use super::piece_view::PieceVisual;
+use super::save_load::SaveLoadState;
 use crate::app::AppScreenState;
 use crate::match_state::{ClaimedDrawReason, MatchSession};
 use crate::style::ShellTheme;
@@ -41,6 +42,9 @@
 struct PromotionHintText;
 
 #[derive(Component)]
+struct PersistenceStatusText;
+
+#[derive(Component)]
 struct ClaimDrawButton;
 
 type HudTextQuery<'w, 's> = Query<
@@ -51,6 +55,7 @@
         Option<&'static TurnStatusText>,
         Option<&'static MatchStatusText>,
         Option<&'static PromotionHintText>,
+        Option<&'static PersistenceStatusText>,
     ),
 >;
 
@@ -61,7 +66,7 @@
                 position_type: PositionType::Absolute,
                 top: Val::Px(24.0),
                 left: Val::Px(24.0),
-                width: Val::Px(360.0),
+                width: Val::Px(380.0),
                 flex_direction: FlexDirection::Column,
                 row_gap: Val::Px(10.0),
                 padding: UiRect::all(Val::Px(18.0)),
@@ -98,6 +103,15 @@
                 TextColor(theme.ui_text),
                 PromotionHintText,
             ));
+            parent.spawn((
+                Text::new("Interrupted-session recovery is waiting for the next autosave."),
+                TextFont {
+                    font_size: 14.0,
+                    ..default()
+                },
+                TextColor(theme.ui_text),
+                PersistenceStatusText,
+            ));
             parent
                 .spawn((
                     Button,
@@ -133,6 +147,7 @@
 
 fn sync_match_hud(
     match_session: Res<MatchSession>,
+    save_state: Res<SaveLoadState>,
     mut text_query: HudTextQuery<'_, '_>,
 ) {
     let turn_label = format!("{} to move", side_label(match_session.game_state().side_to_move()));
@@ -148,14 +163,29 @@
     } else {
         String::from("Promotion uses Q / R / B / N.")
     };
-
-    for (mut text, turn_marker, status_marker, promotion_marker) in &mut text_query {
+    let persistence_label = save_state
+        .last_error
+        .clone()
+        .or_else(|| save_state.last_message.clone())
+        .unwrap_or_else(|| {
+            if match_session.is_recovery_dirty() {
+                String::from("Interrupted-session recovery is waiting for the next autosave.")
+            } else {
+                String::from("Interrupted-session recovery is current.")
+            }
+        });
+
+    for (mut text, turn_marker, status_marker, promotion_marker, persistence_marker) in
+        &mut text_query
+    {
         if turn_marker.is_some() {
             text.0 = turn_label.clone();
         } else if status_marker.is_some() {
             text.0 = status_label.clone();
         } else if promotion_marker.is_some() {
             text.0 = promotion_hint_label.clone();
+        } else if persistence_marker.is_some() {
+            text.0 = persistence_label.clone();
         }
     }
 }

```

**Documentation:**

```diff
--- a/crates/game_app/src/plugins/move_feedback.rs
+++ b/crates/game_app/src/plugins/move_feedback.rs
@@ -41,6 +41,8 @@ struct MatchStatusText;
 struct PromotionHintText;
 
 #[derive(Component)]
+/// Mirrors repository success and error state inside the in-match HUD so recovery freshness stays visible without leaving the board. (ref: DL-003)
 struct PersistenceStatusText;
 
 #[derive(Component)]
@@ -147,6 +149,8 @@ fn spawn_match_hud(mut commands: Commands, theme: Res<ShellTheme>) {
         });
 }
 
+/// Combines chess status with persistence feedback so save/load failures and recovery freshness share the same HUD refresh loop. (ref: DL-003)
 fn sync_match_hud(
     match_session: Res<MatchSession>,
     save_state: Res<SaveLoadState>,

```


**CC-M-002-013** (crates/game_app/tests/match_state_flow.rs) - implements CI-M-002-012

**Code:**

```diff
--- a/crates/game_app/tests/match_state_flow.rs
+++ b/crates/game_app/tests/match_state_flow.rs
@@ -1,99 +1,169 @@+use chess_core::{GameState, Move, Square};
+use chess_persistence::{
+    GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore,
+    ShellSettings, SnapshotMetadata, SnapshotShellState,
+};
+use tempfile::tempdir;
+
 use bevy::prelude::*;
 use bevy::state::app::StatesPlugin;
-use chess_core::{GameState, Square};
-use game_app::{AppScreenState, AppShellPlugin, MatchSession, ShellTheme};
+use game_app::{
+    AppScreenState, AppShellPlugin, BoardScenePlugin, MatchLaunchIntent, MatchSession,
+    MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin, PendingLoadedSnapshot,
+    PieceViewPlugin, SaveLoadPlugin, SaveRootOverride, ShellInputPlugin, ShellMenuState,
+    ShellTheme,
+};
 
-fn test_app() -> App {
+fn test_app(root: &std::path::Path) -> App {
     let mut app = App::new();
     app.add_plugins(MinimalPlugins)
         .add_plugins(StatesPlugin)
+        .insert_resource(Assets::<Mesh>::default())
+        .insert_resource(Assets::<StandardMaterial>::default())
+        .insert_resource(ButtonInput::<KeyCode>::default())
+        .insert_resource(ButtonInput::<MouseButton>::default())
         .insert_resource(ShellTheme::default())
         .insert_resource(MatchSession::start_local_match())
+        .insert_resource(MatchLaunchIntent::default())
+        .insert_resource(PendingLoadedSnapshot::default())
+        .insert_resource(SaveRootOverride(Some(root.to_path_buf())))
         .init_state::<AppScreenState>()
-        .add_plugins(AppShellPlugin);
+        .add_plugins((
+            MenuPlugin,
+            SaveLoadPlugin,
+            AppShellPlugin,
+            BoardScenePlugin,
+            PieceViewPlugin,
+            ShellInputPlugin,
+            MoveFeedbackPlugin,
+        ));
     app
+}
+
+fn bootstrap_shell(app: &mut App) {
+    app.update();
+    app.update();
+}
+
+fn enter_local_match(app: &mut App) {
+    app.world_mut().send_event(MenuAction::OpenSetup);
+    app.update();
+    app.world_mut().send_event(MenuAction::StartNewMatch);
+    app.update();
+    app.update();
+    app.update();
 }
 
 fn current_state(app: &App) -> AppScreenState {
     *app.world().resource::<State<AppScreenState>>().get()
 }
 
+fn tap_key(app: &mut App, key: KeyCode) {
+    app.world_mut()
+        .resource_mut::<ButtonInput<KeyCode>>()
+        .press(key);
+    app.update();
+    app.world_mut()
+        .resource_mut::<ButtonInput<KeyCode>>()
+        .release(key);
+    app.update();
+}
+
+fn sample_snapshot(label: &str) -> GameSnapshot {
+    let game_state =
+        GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").expect("fixture FEN should parse");
+    let from = Square::from_algebraic("e7").expect("valid square");
+    let to = Square::from_algebraic("e8").expect("valid square");
+
+    GameSnapshot::from_parts(
+        game_state,
+        SnapshotMetadata {
+            label: label.to_string(),
+            created_at_utc: Some(String::from("2026-03-15T00:00:00Z")),
+            updated_at_utc: None,
+            notes: None,
+            save_kind: SaveKind::Manual,
+            session_id: label.to_ascii_lowercase().replace(' ', "-"),
+            recovery_key: None,
+        },
+        SnapshotShellState {
+            selected_square: Some(from),
+            pending_promotion: Some(PendingPromotionSnapshot { from, to }),
+            last_move: Some(Move::new(from, to)),
+            claimed_draw: None,
+            dirty_recovery: true,
+        },
+    )
+}
+
 #[test]
-fn match_loading_resets_session_and_enters_in_match() {
-    let mut app = test_app();
+fn manual_load_intent_restores_snapshot_and_enters_in_match() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
 
-    app.update();
-    app.update();
-
-    assert_eq!(current_state(&app), AppScreenState::MainMenu);
-
-    {
-        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
-        match_session.selected_square = Some(Square::from_algebraic("e2").expect("valid square"));
-        match_session.replace_game_state(
-            GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").expect("valid FEN"),
-        );
-    }
-
+    *app.world_mut().resource_mut::<MatchLaunchIntent>() = MatchLaunchIntent::LoadManual;
+    app.world_mut().resource_mut::<PendingLoadedSnapshot>().0 =
+        Some(sample_snapshot("Manual Fixture"));
     app.world_mut()
         .resource_mut::<NextState<AppScreenState>>()
         .set(AppScreenState::MatchLoading);
 
     app.update();
     app.update();
+    app.update();
 
+    assert_eq!(current_state(&app), AppScreenState::InMatch);
     let match_session = app.world().resource::<MatchSession>();
-    assert_eq!(current_state(&app), AppScreenState::InMatch);
-    assert_eq!(match_session.game_state, GameState::starting_position());
-    assert_eq!(match_session.selected_square, None);
-    assert_eq!(match_session.pending_promotion_move, None);
+    assert_eq!(
+        match_session.pending_promotion_move,
+        Some(Move::new(
+            Square::from_algebraic("e7").expect("valid square"),
+            Square::from_algebraic("e8").expect("valid square"),
+        ))
+    );
 }
 
 #[test]
-fn finished_match_reaches_result_then_supports_rematch_and_menu_return() {
-    let mut app = test_app();
+fn escape_opens_setup_overlay_without_leaving_in_match_state() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
+    enter_local_match(&mut app);
 
-    app.update();
+    tap_key(&mut app, KeyCode::Escape);
+
+    assert_eq!(current_state(&app), AppScreenState::InMatch);
+    assert_eq!(app.world().resource::<ShellMenuState>().panel, MenuPanel::Setup);
+
+    app.world_mut().send_event(MenuAction::ReturnToMenu);
     app.update();
 
-    app.world_mut()
-        .resource_mut::<NextState<AppScreenState>>()
-        .set(AppScreenState::MatchLoading);
+    assert_eq!(current_state(&app), AppScreenState::MainMenu);
+}
 
+#[test]
+fn startup_resume_policy_hydrates_recovery_snapshot_through_match_loading() {
+    let root = tempdir().expect("temporary directory should be created");
+    let store = SessionStore::new(root.path());
+    store
+        .store_recovery(sample_snapshot("Recovery Fixture"))
+        .expect("recovery save should succeed");
+    store
+        .save_settings(&ShellSettings {
+            recovery_policy: RecoveryStartupPolicy::Resume,
+            ..ShellSettings::default()
+        })
+        .expect("settings save should succeed");
+
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
     app.update();
-    app.update();
-
-    assert_eq!(current_state(&app), AppScreenState::InMatch);
-
-    app.world_mut()
-        .resource_mut::<MatchSession>()
-        .replace_game_state(
-            GameState::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").expect("valid FEN"),
-        );
-
-    app.update();
-    app.update();
-
-    assert_eq!(current_state(&app), AppScreenState::MatchResult);
-
-    app.world_mut()
-        .resource_mut::<NextState<AppScreenState>>()
-        .set(AppScreenState::MatchLoading);
-
     app.update();
     app.update();
 
     assert_eq!(current_state(&app), AppScreenState::InMatch);
     assert_eq!(
-        app.world().resource::<MatchSession>().game_state,
-        GameState::starting_position()
+        app.world().resource::<MatchSession>().selected_square,
+        Some(Square::from_algebraic("e7").expect("valid square"))
     );
-
-    app.world_mut()
-        .resource_mut::<NextState<AppScreenState>>()
-        .set(AppScreenState::MainMenu);
-
-    app.update();
-
-    assert_eq!(current_state(&app), AppScreenState::MainMenu);
 }
```

**Documentation:**

```diff
--- a/crates/game_app/tests/match_state_flow.rs
+++ b/crates/game_app/tests/match_state_flow.rs
@@ -1,4 +1,7 @@
+//! Integration coverage for launch-intent routing and recovery hydration.
+//! These tests exercise the coarse shell states while verifying snapshots restore legal interaction state. (ref: DL-001) (ref: DL-004)
+
 use chess_core::{GameState, Move, Square};
 use chess_persistence::{
     GameSnapshot, PendingPromotionSnapshot, RecoveryStartupPolicy, SaveKind, SessionStore,
@@ -64,6 +67,8 @@ fn sample_snapshot(label: &str) -> GameSnapshot {
     )
 }
 
+// Manual load still passes through MatchLoading so snapshot restore and route transitions stay coupled. (ref: DL-001) (ref: DL-004)
 #[test]
 fn manual_load_intent_restores_snapshot_and_enters_in_match() {
     let root = tempdir().expect("temporary directory should be created");
@@ -111,6 +116,8 @@ fn escape_opens_setup_overlay_without_leaving_in_match_state() {
     assert_eq!(app.world().resource::<ShellMenuState>().panel, MenuPanel::Setup);
 }
 
+// Startup resume honors the persisted recovery policy instead of inferring behavior from transient UI state. (ref: DL-003) (ref: DL-005)
 #[test]
 fn startup_resume_policy_hydrates_recovery_snapshot_through_match_loading() {
     let root = tempdir().expect("temporary directory should be created");

```


**CC-M-002-014** (crates/game_app/tests/save_load_flow.rs) - implements CI-M-002-013

**Code:**

```diff
--- /dev/null
+++ b/crates/game_app/tests/save_load_flow.rs
@@ -0,0 +1,193 @@+use chess_core::{GameState, Move, PieceKind, Side, Square};
+use chess_persistence::{DisplayMode, RecoveryStartupPolicy};
+use tempfile::tempdir;
+
+use bevy::prelude::*;
+use bevy::state::app::StatesPlugin;
+use game_app::{
+    AppScreenState, AppShellPlugin, BoardScenePlugin, MatchLaunchIntent, MatchSession,
+    MenuAction, MenuPanel, MenuPlugin, MoveFeedbackPlugin, PendingLoadedSnapshot,
+    PieceViewPlugin, PieceVisual, RecoveryBannerState, SaveLoadPlugin, SaveLoadRequest,
+    SaveLoadState, SaveRootOverride, ShellInputPlugin, ShellMenuState, ShellTheme,
+};
+
+fn test_app(root: &std::path::Path) -> App {
+    let mut app = App::new();
+    app.add_plugins(MinimalPlugins)
+        .add_plugins(StatesPlugin)
+        .insert_resource(Assets::<Mesh>::default())
+        .insert_resource(Assets::<StandardMaterial>::default())
+        .insert_resource(ButtonInput::<KeyCode>::default())
+        .insert_resource(ButtonInput::<MouseButton>::default())
+        .insert_resource(ShellTheme::default())
+        .insert_resource(MatchSession::start_local_match())
+        .insert_resource(MatchLaunchIntent::default())
+        .insert_resource(PendingLoadedSnapshot::default())
+        .insert_resource(SaveRootOverride(Some(root.to_path_buf())))
+        .init_state::<AppScreenState>()
+        .add_plugins((
+            MenuPlugin,
+            SaveLoadPlugin,
+            AppShellPlugin,
+            BoardScenePlugin,
+            PieceViewPlugin,
+            ShellInputPlugin,
+            MoveFeedbackPlugin,
+        ));
+    app
+}
+
+fn bootstrap_shell(app: &mut App) {
+    app.update();
+    app.update();
+}
+
+fn enter_local_match(app: &mut App) {
+    app.world_mut().send_event(MenuAction::OpenSetup);
+    app.update();
+    app.world_mut().send_event(MenuAction::StartNewMatch);
+    app.update();
+    app.update();
+    app.update();
+}
+
+fn current_state(app: &App) -> AppScreenState {
+    *app.world().resource::<State<AppScreenState>>().get()
+}
+
+fn tap_key(app: &mut App, key: KeyCode) {
+    app.world_mut()
+        .resource_mut::<ButtonInput<KeyCode>>()
+        .press(key);
+    app.update();
+    app.world_mut()
+        .resource_mut::<ButtonInput<KeyCode>>()
+        .release(key);
+    app.update();
+}
+
+fn piece_visuals(app: &mut App) -> Vec<PieceVisual> {
+    let world = app.world_mut();
+    let mut query = world.query::<&PieceVisual>();
+    query.iter(world).copied().collect()
+}
+
+#[test]
+fn manual_save_and_load_roundtrip_restores_pending_promotion() {
+    let root = tempdir().expect("temporary directory should be created");
+    let mut app = test_app(root.path());
+    bootstrap_shell(&mut app);
+    enter_local_match(&mut app);
+
+    let promotion_from = Square::from_algebraic("e7").expect("valid square");
+    let promotion_to = Square::from_algebraic("e8").expect("valid square");
+    {
+        let mut match_session = app.world_mut().resource_mut::<MatchSession>();
+        match_session.replace_game_state(
+            GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1")
+                .expect("fixture FEN should parse"),
+        );
+        match_session.selected_square = Some(promotion_from);
+        match_session.pending_promotion_move = Some(Move::new(promotion_from, promotion_to));
+        match_session.mark_recovery_dirty();
+    }
+    app.update();
+
+    tap_key(&mut app, KeyCode::F5);
+    assert_eq!(app.world().resource::<SaveLoadState>().manual_saves.len(), 1);
+
+    let slot_id = app.world().resource::<SaveLoadState>().manual_saves[0]
+        .slot_id
+        .clone();
+
+    app.world_mut()
+        .resource_mut::<MatchSession>()
+        .reset_for_local_match();
+    app.update();
+
+    app.world_mut()
+        .send_event(SaveLoadRequest::LoadManual { slot_id });
+    app.update();
+    app.update();
+    app.update();
+
+    assert_eq!(current_state(&app), AppScreenState::InMatch);
+    let match_session = app.world().resource::<MatchSession>();
+    assert_eq!(
+        match_session.pending_promotion_move,
+        Some(Move::new(promotion_from, promotion_to))
+    );
+    assert_eq!(
+        match_session.game_state,
+        GameState::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1")
+            .expect("fixture FEN should parse")
+    );
+
+    let piece_visuals = piece_visuals(&mut app);
+    assert!(piece_visuals.iter().any(|piece_visual| {
+        piece_visual.square == promotion_from
+            && piece_visual.piece.kind == PieceKind::Pawn
+            && piece_visual.piece.side == Side::White
+    }));
+}
+
+#[test]
+fn keyboard_pause_overlay_and_recovery_settings_survive_restart() {
+    let root = tempdir().expect("temporary directory should be created");
+    let expected_state =
+        GameState::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").expect("fixture FEN should parse");
+    let selected = Square::from_algebraic("e2").expect("valid square");
+
+    {
+        let mut app = test_app(root.path());
+        bootstrap_shell(&mut app);
+        enter_local_match(&mut app);
+
+        {
+            let mut match_session = app.world_mut().resource_mut::<MatchSession>();
+            match_session.replace_game_state(expected_state.clone());
+            match_session.selected_square = Some(selected);
+            match_session.mark_recovery_dirty();
+        }
+        app.update();
+
+        tap_key(&mut app, KeyCode::Escape);
+        assert_eq!(current_state(&app), AppScreenState::InMatch);
+        assert_eq!(app.world().resource::<ShellMenuState>().panel, MenuPanel::Setup);
+
+        {
+            let mut save_state = app.world_mut().resource_mut::<SaveLoadState>();
+            save_state.settings.display_mode = DisplayMode::Fullscreen;
+            save_state.settings.recovery_policy = RecoveryStartupPolicy::Ask;
+        }
+        app.world_mut().send_event(SaveLoadRequest::PersistSettings);
+        app.update();
+    }
+
+    let mut restarted = test_app(root.path());
+    bootstrap_shell(&mut restarted);
+
+    let save_state = restarted.world().resource::<SaveLoadState>();
+    assert_eq!(save_state.settings.display_mode, DisplayMode::Fullscreen);
+    assert_eq!(
+        save_state.settings.recovery_policy,
+        RecoveryStartupPolicy::Ask
+    );
+    assert!(
+        restarted
+            .world()
+            .resource::<RecoveryBannerState>()
+            .available
+    );
+
+    restarted
+        .world_mut()
+        .send_event(SaveLoadRequest::ResumeRecovery);
+    restarted.update();
+    restarted.update();
+    restarted.update();
+
+    assert_eq!(current_state(&restarted), AppScreenState::InMatch);
+    let match_session = restarted.world().resource::<MatchSession>();
+    assert_eq!(match_session.game_state, expected_state);
+    assert_eq!(match_session.selected_square, Some(selected));
+}
```

**Documentation:**

```diff
--- a/crates/game_app/tests/save_load_flow.rs
+++ b/crates/game_app/tests/save_load_flow.rs
@@ -0,0 +1,4 @@
+//! Integration coverage for manual save/load, pause overlay, and persisted settings.
+//! The shell tests lock down startup recovery and display mode while keeping destructive confirmations inside the same persisted settings contract. (ref: DL-003) (ref: DL-004) (ref: DL-005)
+
 use chess_core::{GameState, Move, PieceKind, Side, Square};
@@ -66,6 +70,8 @@ fn piece_visuals(app: &mut App) -> Vec<PieceVisual> {
     query.iter(world).copied().collect()
 }
 
+// Manual save/load restores pending promotion because legality-critical shell state lives inside the snapshot contract. (ref: DL-004)
 #[test]
 fn manual_save_and_load_roundtrip_restores_pending_promotion() {
     let root = tempdir().expect("temporary directory should be created");
@@ -121,6 +127,8 @@ fn manual_save_and_load_roundtrip_restores_pending_promotion() {
     }));
 }
 
+// Restart coverage exercises startup recovery and display mode while keeping destructive confirmations on the same repository-backed settings boundary rather than transient app state. (ref: DL-005) (ref: DL-008)
 #[test]
 fn keyboard_pause_overlay_and_recovery_settings_survive_restart() {
     let root = tempdir().expect("temporary directory should be created");

```


**CC-M-002-015** (Cargo.lock) - implements CI-M-002-014

**Code:**

```diff
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -2405,6 +2405,7 @@
  "chess_core",
  "chess_persistence",
  "engine_uci",
+ "tempfile",
 ]
 
 [[package]]
```

**Documentation:**

```diff
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -2403,7 +2403,8 @@
+# Shell integration tests add `tempfile` for injected roots while runtime storage still resolves through SessionStore. (ref: DL-008)
 dependencies = [
  "bevy",
  "chess_core",
  "chess_persistence",
  "engine_uci",
  "tempfile",
 ]
 

```


### Milestone 3: CI Artifact Packaging and Boot Smoke

**Files**: .github/workflows/ci.yml, tools/ci/package-game-app.sh, tools/ci/package-game-app.ps1, tools/ci/smoke-boot-linux.sh, tools/ci/smoke-boot-windows.ps1

**Flags**: ci, artifacts, smoke

**Requirements**:

- Package portable Windows and Linux game_app builds as downloadable CI artifacts
- Run scripted startup smoke checks against the packaged outputs on both operating systems
- Keep existing fmt clippy test and release-build lanes intact while adding artifact publication

**Acceptance Criteria**:

- The workflow uploads bootable Windows and Linux artifact archives
- Startup smoke checks fail the job when the packaged binary crashes before the timeout window
- The pipeline remains installer-free and runnable on GitHub-hosted runners

**Tests**:

- ci: packaged binary startup smoke on linux
- ci: packaged binary startup smoke on windows

#### Code Intent

- **CI-M-003-001** `.github/workflows/ci.yml::GitHub Actions workflow`: Extend the release job to assemble upload and smoke-test portable game_app artifacts for Windows and Linux while preserving the existing format lint and test matrix. (refs: DL-006)
- **CI-M-003-002** `tools/ci/package-game-app.sh::linux artifact assembly`: Create the Linux artifact layout and archive around the release game_app binary and any runtime files the packaged build needs. (refs: DL-006)
- **CI-M-003-003** `tools/ci/package-game-app.ps1::windows artifact assembly`: Create the Windows artifact layout and archive around the release game_app executable and any runtime files the packaged build needs. (refs: DL-006)
- **CI-M-003-004** `tools/ci/smoke-boot-linux.sh::linux startup smoke`: Launch the packaged Linux build under a bounded timeout and fail fast when the process exits before successful startup. (refs: DL-006)
- **CI-M-003-005** `tools/ci/smoke-boot-windows.ps1::windows startup smoke`: Launch the packaged Windows build under a bounded timeout and report startup failure when the process crashes or exits too early. (refs: DL-006)

#### Code Changes

**CC-M-003-001** (.github/workflows/ci.yml) - implements CI-M-003-001

**Code:**

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -156,6 +156,33 @@
             libxkbcommon-dev \
             libxrandr-dev \
             libxxf86vm-dev \
-            pkg-config
+            pkg-config \
+            xvfb
       - name: Build release workspace
         run: cargo build --workspace --release
+      - name: Package Linux artifact
+        if: runner.os == 'Linux'
+        run: bash tools/ci/package-game-app.sh "$GITHUB_WORKSPACE" "$RUNNER_TEMP/artifacts" "game_app-linux-x86_64"
+      - name: Package Windows artifact
+        if: runner.os == 'Windows'
+        shell: pwsh
+        run: tools/ci/package-game-app.ps1 -WorkspaceRoot $env:GITHUB_WORKSPACE -DistDir "$env:RUNNER_TEMP/artifacts" -ArtifactName "game_app-windows-x86_64"
+      - name: Smoke boot Linux artifact
+        if: runner.os == 'Linux'
+        run: bash tools/ci/smoke-boot-linux.sh "$RUNNER_TEMP/artifacts/game_app-linux-x86_64.tar.gz"
+      - name: Smoke boot Windows artifact
+        if: runner.os == 'Windows'
+        shell: pwsh
+        run: tools/ci/smoke-boot-windows.ps1 -ArchivePath "$env:RUNNER_TEMP/artifacts/game_app-windows-x86_64.zip"
+      - name: Upload Linux artifact
+        if: runner.os == 'Linux'
+        uses: actions/upload-artifact@v4
+        with:
+          name: game_app-linux-x86_64
+          path: ${{ runner.temp }}/artifacts/game_app-linux-x86_64.tar.gz
+      - name: Upload Windows artifact
+        if: runner.os == 'Windows'
+        uses: actions/upload-artifact@v4
+        with:
+          name: game_app-windows-x86_64
+          path: ${{ runner.temp }}/artifacts/game_app-windows-x86_64.zip
```

**Documentation:**

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -156,6 +156,7 @@
             libxxf86vm-dev \
             pkg-config \
             xvfb
+      # Packaging, boot smoke, and upload together prove portable runtime output rather than compile-only success. (ref: DL-006)
       - name: Build release workspace
         run: cargo build --workspace --release
       - name: Package Linux artifact

```


**CC-M-003-002** (tools/ci/package-game-app.sh) - implements CI-M-003-002

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/package-game-app.sh
@@ -0,0 +1,19 @@
+#!/usr/bin/env bash
+set -euo pipefail
+
+workspace_root="${1:?workspace root required}"
+dist_dir="${2:?dist dir required}"
+artifact_name="${3:-game_app-linux-x86_64}"
+binary_path="${workspace_root}/target/release/game_app"
+staging_dir="${dist_dir}/${artifact_name}"
+archive_path="${dist_dir}/${artifact_name}.tar.gz"
+
+# Portable archives keep M3 installer-free while still shipping everything the app boots with.
+rm -rf "${staging_dir}" "${archive_path}"
+mkdir -p "${staging_dir}"
+
+cp "${binary_path}" "${staging_dir}/game_app"
+cp -R "${workspace_root}/assets" "${staging_dir}/assets"
+
+tar -C "${dist_dir}" -czf "${archive_path}" "${artifact_name}"
+printf '%s\n' "${archive_path}"
```

**Documentation:**

```diff
--- a/tools/ci/package-game-app.sh
+++ b/tools/ci/package-game-app.sh
@@ -1,5 +1,8 @@
 #!/usr/bin/env bash
 set -euo pipefail
+
+# Produces a portable Linux bundle that includes the game binary plus runtime assets. (ref: DL-006)
+# CI ships archives and smoke-tests the extracted app directory as the packaged-boot contract. (ref: DL-006)
 
 workspace_root="${1:?workspace root required}"
 dist_dir="${2:?dist dir required}"
@@ -7,6 +10,7 @@ artifact_name="${3:-game_app-linux-x86_64}"
 binary_path="${workspace_root}/target/release/game_app"
 staging_dir="${dist_dir}/${artifact_name}"
 archive_path="${dist_dir}/${artifact_name}.tar.gz"
 
+# The archive layout keeps a single top-level app directory so smoke scripts can locate the runnable package deterministically. (ref: DL-006)
+# Portable archives keep the staged app directory self-contained with every runtime file the binary expects at boot. (ref: DL-006)
-# Portable archives keep M3 installer-free while still shipping everything the app boots with.
 rm -rf "${staging_dir}" "${archive_path}"
 mkdir -p "${staging_dir}"

```


**CC-M-003-003** (tools/ci/package-game-app.ps1) - implements CI-M-003-003

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/package-game-app.ps1
@@ -0,0 +1,23 @@
+param(
+    [Parameter(Mandatory = $true)]
+    [string]$WorkspaceRoot,
+    [Parameter(Mandatory = $true)]
+    [string]$DistDir,
+    [string]$ArtifactName = "game_app-windows-x86_64"
+)
+
+$BinaryPath = Join-Path $WorkspaceRoot "target/release/game_app.exe"
+$StagingDir = Join-Path $DistDir $ArtifactName
+$ArchivePath = Join-Path $DistDir ("{0}.zip" -f $ArtifactName)
+
+# Portable archives keep M3 focused on bootable artifacts instead of installer plumbing.
+New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
+Remove-Item $StagingDir -Recurse -Force -ErrorAction SilentlyContinue
+Remove-Item $ArchivePath -Force -ErrorAction SilentlyContinue
+New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null
+
+Copy-Item -Path $BinaryPath -Destination (Join-Path $StagingDir "game_app.exe")
+Copy-Item -Path (Join-Path $WorkspaceRoot "assets") -Destination (Join-Path $StagingDir "assets") -Recurse
+
+Compress-Archive -Path $StagingDir -DestinationPath $ArchivePath -Force
+Write-Output $ArchivePath
```

**Documentation:**

```diff
--- a/tools/ci/package-game-app.ps1
+++ b/tools/ci/package-game-app.ps1
@@ -1,6 +1,8 @@
 param(
     [Parameter(Mandatory = $true)]
     [string]$WorkspaceRoot,
@@ -5,6 +7,8 @@ param(
     [string]$ArtifactName = "game_app-windows-x86_64"
 )
 
+# Produces a portable Windows bundle that keeps the extracted app directory self-contained for smoke startup checks. (ref: DL-006)
+
 $BinaryPath = Join-Path $WorkspaceRoot "target/release/game_app.exe"
 $StagingDir = Join-Path $DistDir $ArtifactName
 $ArchivePath = Join-Path $DistDir ("{0}.zip" -f $ArtifactName)
@@ -11,6 +15,7 @@ $ArchivePath = Join-Path $DistDir ("{0}.zip" -f $ArtifactName)
 
+# The staged directory survives zip extraction as the single runnable package root. (ref: DL-006)
+# Portable archives keep the staged app directory self-contained for direct extraction and smoke startup checks. (ref: DL-006)
-# Portable archives keep M3 focused on bootable artifacts instead of installer plumbing.
 New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
 Remove-Item $StagingDir -Recurse -Force -ErrorAction SilentlyContinue

```


**CC-M-003-004** (tools/ci/smoke-boot-linux.sh) - implements CI-M-003-004

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/smoke-boot-linux.sh
@@ -0,0 +1,34 @@
+#!/usr/bin/env bash
+set -euo pipefail
+
+archive_path="${1:?archive path required}"
+smoke_dir="$(mktemp -d)"
+log_path="${smoke_dir}/game_app.log"
+trap 'rm -rf "${smoke_dir}"' EXIT
+
+tar -xzf "${archive_path}" -C "${smoke_dir}"
+app_dir="$(find "${smoke_dir}" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
+if [[ -z "${app_dir}" || ! -x "${app_dir}/game_app" ]]; then
+    echo "packaged linux archive did not extract to a runnable app directory"
+    exit 1
+fi
+
+pushd "${app_dir}" >/dev/null
+# A timeout-driven pass proves the packaged binary stayed alive long enough to finish startup.
+set +e
+timeout 15s xvfb-run -a env WGPU_BACKEND=gl ./game_app >"${log_path}" 2>&1
+status="$?"
+set -e
+
+if [[ "${status}" -eq 0 ]]; then
+    cat "${log_path}"
+    echo "game_app exited before the smoke timeout"
+    exit 1
+fi
+
+if [[ "${status}" -ne 124 ]]; then
+    cat "${log_path}"
+    echo "game_app failed during packaged startup smoke"
+    exit "${status}"
+fi
+popd >/dev/null
```

**Documentation:**

```diff
--- a/tools/ci/smoke-boot-linux.sh
+++ b/tools/ci/smoke-boot-linux.sh
@@ -1,5 +1,8 @@
 #!/usr/bin/env bash
 set -euo pipefail
+
+# Extracts the packaged Linux archive and treats surviving the timeout window as proof of a bootable artifact. (ref: DL-006)
 
 archive_path="${1:?archive path required}"
 smoke_dir="$(mktemp -d)"
@@ -14,6 +17,7 @@ if [[ -z "${app_dir}" || ! -x "${app_dir}/game_app" ]]; then
 fi
 
 pushd "${app_dir}" >/dev/null
+# xvfb-run provides a stable desktop surface on GitHub-hosted runners so boot validation measures packaged startup instead of compile-only success. (ref: DL-006)
 # A timeout-driven pass proves the packaged binary stayed alive long enough to finish startup.
 set +e
 timeout 15s xvfb-run -a env WGPU_BACKEND=gl ./game_app >"${log_path}" 2>&1

```


**CC-M-003-005** (tools/ci/smoke-boot-windows.ps1) - implements CI-M-003-005

**Code:**

```diff
--- /dev/null
+++ b/tools/ci/smoke-boot-windows.ps1
@@ -0,0 +1,30 @@
+param(
+    [Parameter(Mandatory = $true)]
+    [string]$ArchivePath,
+    [int]$StartupSeconds = 10
+)
+
+$SmokeDir = Join-Path $env:RUNNER_TEMP ("game-app-smoke-{0}" -f [guid]::NewGuid().ToString("N"))
+New-Item -ItemType Directory -Path $SmokeDir -Force | Out-Null
+try {
+    Expand-Archive -Path $ArchivePath -DestinationPath $SmokeDir -Force
+    $AppDir = Get-ChildItem $SmokeDir -Directory |
+        Where-Object { Test-Path (Join-Path $_.FullName "game_app.exe") } |
+        Select-Object -First 1
+    if (-not $AppDir) {
+        throw "archive did not extract to a packaged app directory containing game_app.exe"
+    }
+    $ExePath = Join-Path $AppDir.FullName "game_app.exe"
+
+    # Staying alive through the timeout window is the M3 proof that the packaged app booted.
+    $Process = Start-Process -FilePath $ExePath -WorkingDirectory $AppDir.FullName -PassThru
+    Start-Sleep -Seconds $StartupSeconds
+    if ($Process.HasExited) {
+        throw "game_app exited early with code $($Process.ExitCode)"
+    }
+
+    Stop-Process -Id $Process.Id -Force
+}
+finally {
+    Remove-Item $SmokeDir -Recurse -Force -ErrorAction SilentlyContinue
+}
```

**Documentation:**

```diff
--- a/tools/ci/smoke-boot-windows.ps1
+++ b/tools/ci/smoke-boot-windows.ps1
@@ -1,6 +1,8 @@
 param(
     [Parameter(Mandatory = $true)]
     [string]$ArchivePath,
@@ -3,6 +5,8 @@ param(
     [int]$StartupSeconds = 10
 )
 
+# Extracts the packaged Windows archive and treats a process that survives the startup window as a bootable artifact. (ref: DL-006)
+
 $SmokeDir = Join-Path $env:RUNNER_TEMP ("game-app-smoke-{0}" -f [guid]::NewGuid().ToString("N"))
 New-Item -ItemType Directory -Path $SmokeDir -Force | Out-Null
 try {
@@ -17,6 +21,7 @@ try {
     }
     $ExePath = Join-Path $AppDir.FullName "game_app.exe"
 
+    # The timeout window checks packaged startup on the runner from the extracted app directory itself. (ref: DL-006)
     # Staying alive through the timeout window is the M3 proof that the packaged app booted.
     $Process = Start-Process -FilePath $ExePath -WorkingDirectory $AppDir.FullName -PassThru
     Start-Sleep -Seconds $StartupSeconds

```


**CC-M-003-006** (tools/ci/README.md)

**Documentation:**

```diff
--- /dev/null
+++ b/tools/ci/README.md
@@ -0,0 +1,18 @@
+# CI Packaging Notes
+
+Portable artifact packaging and smoke-start verification for `game_app`.
+
+## Architecture
+
+- Packaging scripts stage the release binary with the runtime `assets/` tree into a single top-level app directory.
+- Smoke scripts extract that directory and treat surviving a bounded startup window as proof of a bootable artifact. (ref: DL-006)
+
+## Invariants
+
+- CI proves packaged runtime boot, not just successful compilation. (ref: DL-006)
+- Windows and Linux archives stay self-contained with one top-level app directory so the workflow can upload runnable artifacts. (ref: DL-006)
+
+## Runner Expectations
+
+- Linux smoke uses `xvfb-run` to provide a stable desktop surface on hosted runners.
+- Windows smoke starts the extracted executable directly from the staged app directory.

```


## Execution Waves

- W-001: M-001
- W-002: M-002, M-003
