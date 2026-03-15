//! File-backed repository for manual saves, interrupted-session recovery, and the shipped shell settings trio.
//! The repository owns platform paths and atomic I/O so gameplay code only exchanges snapshots.

use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::{GameSnapshot, SaveKind};

const APP_DATA_QUALIFIER: &str = "io";
const APP_DATA_ORG: &str = "franky";
const APP_DATA_NAME: &str = "3d-chess";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RecoveryStartupPolicy {
    Resume,
    Ignore,
    #[default]
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DisplayMode {
    #[default]
    Windowed,
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmActionSettings {
    pub overwrite_save: bool,
    pub delete_save: bool,
    pub abandon_match: bool,
}

impl Default for ConfirmActionSettings {
    fn default() -> Self {
        Self {
            overwrite_save: true,
            delete_save: true,
            abandon_match: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ShellSettings {
    pub recovery_policy: RecoveryStartupPolicy,
    pub confirm_actions: ConfirmActionSettings,
    pub display_mode: DisplayMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSessionSummary {
    pub slot_id: String,
    pub label: String,
    pub created_at_utc: Option<String>,
    pub save_kind: SaveKind,
}

impl SavedSessionSummary {
    #[must_use]
    pub fn from_snapshot(snapshot: &GameSnapshot) -> Self {
        Self::from_snapshot_with_slot_id(snapshot.metadata.session_id.clone(), snapshot)
    }

    #[must_use]
    fn from_snapshot_with_slot_id(slot_id: String, snapshot: &GameSnapshot) -> Self {
        Self {
            slot_id,
            label: snapshot.metadata.label.clone(),
            created_at_utc: snapshot.metadata.created_at_utc.clone(),
            save_kind: snapshot.metadata.save_kind,
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Serialization(serde_json::Error),
    MissingPlatformDir,
    InvalidSlotId(String),
}

impl Display for StoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Serialization(error) => write!(formatter, "serialization error: {error}"),
            Self::MissingPlatformDir => formatter.write_str("missing platform data directory"),
            Self::InvalidSlotId(slot_id) => write!(formatter, "invalid slot id: {slot_id}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn runtime() -> StoreResult<Self> {
        Ok(Self::new(Self::default_root()?))
    }

    pub fn default_root() -> StoreResult<PathBuf> {
        let Some(project_dirs) = ProjectDirs::from(APP_DATA_QUALIFIER, APP_DATA_ORG, APP_DATA_NAME)
        else {
            return Err(StoreError::MissingPlatformDir);
        };

        Ok(project_dirs.data_dir().to_path_buf())
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list_manual_saves(&self) -> StoreResult<Vec<SavedSessionSummary>> {
        self.ensure_layout()?;
        let mut saves = Vec::new();
        for entry in fs::read_dir(self.manual_saves_dir())? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let Some(slot_id) = manual_slot_id_from_path(&entry.path())? else {
                continue;
            };
            let snapshot: GameSnapshot = self.read_json(&entry.path())?;
            saves.push(SavedSessionSummary {
                slot_id,
                label: snapshot.metadata.label.clone(),
                created_at_utc: snapshot.metadata.created_at_utc.clone(),
                save_kind: SaveKind::Manual,
            });
        }

        saves.sort_by(|left, right| right.created_at_utc.cmp(&left.created_at_utc));
        Ok(saves)
    }

    pub fn save_manual(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
        self.ensure_layout()?;
        let now = now_utc();
        if snapshot.metadata.label.trim().is_empty() {
            snapshot.metadata.label = format!("Manual Save {now}");
        }
        snapshot.metadata.session_id = if snapshot.metadata.session_id.trim().is_empty() {
            self.next_manual_slot_id(&snapshot.metadata.label)?
        } else {
            validate_slot_id(&snapshot.metadata.session_id)?
        };
        snapshot.metadata.save_kind = SaveKind::Manual;
        if snapshot.metadata.created_at_utc.is_none() {
            snapshot.metadata.created_at_utc = Some(now.clone());
        }
        snapshot.metadata.updated_at_utc = Some(now);

        let summary = SavedSessionSummary::from_snapshot(&snapshot);
        self.write_json_atomic(&self.manual_save_path(&summary.slot_id)?, &snapshot)?;
        Ok(summary)
    }

    pub fn load_manual(&self, slot_id: &str) -> StoreResult<GameSnapshot> {
        let slot_id = validate_slot_id(slot_id)?;
        let mut snapshot: GameSnapshot =
            self.read_json(&self.manual_save_path_unchecked(&slot_id))?;
        snapshot.metadata.session_id = slot_id;
        snapshot.metadata.save_kind = SaveKind::Manual;
        Ok(snapshot)
    }

    pub fn delete_manual(&self, slot_id: &str) -> StoreResult<()> {
        let path = self.manual_save_path(slot_id)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn store_recovery(&self, mut snapshot: GameSnapshot) -> StoreResult<SavedSessionSummary> {
        self.ensure_layout()?;
        let now = now_utc();
        snapshot.metadata.save_kind = SaveKind::Recovery;
        snapshot.metadata.session_id = String::from("recovery");
        snapshot
            .metadata
            .recovery_key
            .get_or_insert_with(|| String::from("autosave"));
        if snapshot.metadata.created_at_utc.is_none() {
            snapshot.metadata.created_at_utc = Some(now.clone());
        }
        snapshot.metadata.updated_at_utc = Some(now);

        let summary = SavedSessionSummary::from_snapshot(&snapshot);
        self.write_json_atomic(&self.recovery_file(), &snapshot)?;
        Ok(summary)
    }

    pub fn load_recovery(&self) -> StoreResult<Option<GameSnapshot>> {
        let path = self.recovery_file();
        if !path.exists() {
            return Ok(None);
        }

        let mut snapshot: GameSnapshot = self.read_json(&path)?;
        snapshot.metadata.session_id = String::from("recovery");
        snapshot.metadata.save_kind = SaveKind::Recovery;
        Ok(Some(snapshot))
    }

    pub fn clear_recovery(&self) -> StoreResult<()> {
        let path = self.recovery_file();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn load_settings(&self) -> StoreResult<ShellSettings> {
        let path = self.settings_file();
        if !path.exists() {
            return Ok(ShellSettings::default());
        }

        self.read_json(&path)
    }

    pub fn save_settings(&self, settings: &ShellSettings) -> StoreResult<()> {
        self.ensure_layout()?;
        self.write_json_atomic(&self.settings_file(), settings)
    }

    fn ensure_layout(&self) -> StoreResult<()> {
        fs::create_dir_all(self.manual_saves_dir())?;
        fs::create_dir_all(self.recovery_dir())?;
        Ok(())
    }

    fn next_manual_slot_id(&self, label: &str) -> StoreResult<String> {
        let base = slugify_label(label);
        let base = if base.is_empty() {
            String::from("manual-save")
        } else {
            base
        };
        let mut candidate = base.clone();
        let mut suffix = 2;

        while self.manual_save_path_unchecked(&candidate).exists() {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }

        Ok(candidate)
    }

    fn manual_saves_dir(&self) -> PathBuf {
        self.root.join("saves")
    }

    fn manual_save_path(&self, slot_id: &str) -> StoreResult<PathBuf> {
        Ok(self.manual_save_path_unchecked(&validate_slot_id(slot_id)?))
    }

    fn manual_save_path_unchecked(&self, slot_id: &str) -> PathBuf {
        self.manual_saves_dir().join(format!("{slot_id}.json"))
    }

    fn recovery_dir(&self) -> PathBuf {
        self.root.join("recovery")
    }

    fn recovery_file(&self) -> PathBuf {
        self.recovery_dir().join("current.json")
    }

    fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> StoreResult<T> {
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn write_json_atomic<T: Serialize>(&self, path: &Path, value: &T) -> StoreResult<()> {
        let Some(parent) = path.parent() else {
            return Err(StoreError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path must have a parent directory",
            )));
        };

        fs::create_dir_all(parent)?;
        let mut temp_file = NamedTempFile::new_in(parent)?;
        let bytes = serde_json::to_vec_pretty(value)?;
        temp_file.write_all(&bytes)?;
        temp_file.flush()?;
        temp_file
            .persist(path)
            .map_err(|error| StoreError::Io(error.error))?;
        Ok(())
    }
}

fn now_utc() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 timestamp formatting should be infallible")
}

fn slugify_label(label: &str) -> String {
    label
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            'A'..='Z' => character.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn validate_slot_id(slot_id: &str) -> StoreResult<String> {
    let trimmed = slot_id.trim();
    if trimmed.is_empty()
        || !trimmed
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
    {
        return Err(StoreError::InvalidSlotId(slot_id.to_string()));
    }

    Ok(trimmed.to_string())
}

fn manual_slot_id_from_path(path: &Path) -> StoreResult<Option<String>> {
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return Ok(None);
    }

    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return Err(StoreError::InvalidSlotId(path.display().to_string()));
    };

    Ok(Some(validate_slot_id(stem)?))
}
