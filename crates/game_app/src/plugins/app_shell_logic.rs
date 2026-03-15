use chess_core::{AutomaticDrawReason, DrawReason, GameOutcome, GameStatus, Move, Side, WinReason};
use chess_persistence::{DisplayMode, RecoveryStartupPolicy, SavedSessionSummary};

use crate::app::AppScreenState;
use crate::match_state::ClaimedDrawReason;
use crate::plugins::menu::{ConfirmationKind, MenuContext, RecoveryBannerState, ShellMenuState};
use crate::plugins::save_load::SaveLoadState;

pub fn return_to_menu_abandons_active_match(
    state: AppScreenState,
    menu_state: &ShellMenuState,
) -> bool {
    state == AppScreenState::InMatch && menu_state.context == MenuContext::InMatchOverlay
}

pub fn effective_shell_status(
    menu_state: &ShellMenuState,
    save_state: &SaveLoadState,
    recovery: &RecoveryBannerState,
) -> Option<String> {
    save_state
        .last_error
        .clone()
        .or_else(|| save_state.last_message.clone())
        .or_else(|| menu_state.status_line.clone())
        .or_else(|| {
            if recovery.available {
                recovery
                    .label
                    .as_ref()
                    .map(|label| format!("Interrupted-session recovery is available as {label}."))
            } else {
                None
            }
        })
}

pub fn derive_save_label(last_move: Option<Move>) -> String {
    if let Some(last_move) = last_move {
        format!("Local Match after {last_move}")
    } else {
        String::from("Local Match Save")
    }
}

pub fn selected_save_summary<'a>(
    menu_state: &ShellMenuState,
    save_state: &'a SaveLoadState,
) -> Option<&'a SavedSessionSummary> {
    let slot_id = menu_state.selected_save.as_deref()?;
    save_state
        .manual_saves
        .iter()
        .find(|summary| summary.slot_id == slot_id)
}

pub fn next_recovery_policy(current: RecoveryStartupPolicy) -> RecoveryStartupPolicy {
    match current {
        RecoveryStartupPolicy::Resume => RecoveryStartupPolicy::Ask,
        RecoveryStartupPolicy::Ask => RecoveryStartupPolicy::Ignore,
        RecoveryStartupPolicy::Ignore => RecoveryStartupPolicy::Resume,
    }
}

pub fn recovery_policy_label(policy: RecoveryStartupPolicy) -> &'static str {
    match policy {
        RecoveryStartupPolicy::Resume => "Resume automatically",
        RecoveryStartupPolicy::Ask => "Ask on startup",
        RecoveryStartupPolicy::Ignore => "Ignore recovery on startup",
    }
}

pub fn display_mode_label(mode: DisplayMode) -> &'static str {
    match mode {
        DisplayMode::Windowed => "Windowed",
        DisplayMode::Fullscreen => "Fullscreen",
    }
}

pub fn toggle_label(label: &str, enabled: bool) -> String {
    if enabled {
        format!("{label}: on")
    } else {
        format!("{label}: off")
    }
}

pub fn confirmation_copy(kind: ConfirmationKind) -> (&'static str, &'static str) {
    match kind {
        ConfirmationKind::AbandonMatch => (
            "Leave the current match?",
            "Clearing the recovery slot prevents startup resume from restoring this position.",
        ),
        ConfirmationKind::DeleteSave => (
            "Delete the selected save?",
            "Manual save history is user-controlled so deletes stay explicit.",
        ),
        ConfirmationKind::OverwriteSave => (
            "Overwrite the selected save?",
            "Manual saves stay distinct from recovery, so overwrites should always be deliberate.",
        ),
    }
}

pub fn match_session_result_title(
    status: GameStatus,
    claimed_draw: Option<ClaimedDrawReason>,
) -> String {
    if let Some(claimed_draw_reason) = claimed_draw {
        return match claimed_draw_reason {
            ClaimedDrawReason::ThreefoldRepetition => String::from("Draw Claimed by Repetition"),
            ClaimedDrawReason::FiftyMoveRule => String::from("Draw Claimed by Fifty-Move Rule"),
        };
    }

    match status {
        GameStatus::Ongoing { .. } => String::from("Match Complete"),
        GameStatus::Finished(GameOutcome::Win {
            winner: Side::White,
            reason: WinReason::Checkmate,
        }) => String::from("White Wins"),
        GameStatus::Finished(GameOutcome::Win {
            winner: Side::Black,
            reason: WinReason::Checkmate,
        }) => String::from("Black Wins"),
        GameStatus::Finished(GameOutcome::Draw(_)) => String::from("Draw"),
    }
}

pub fn match_session_result_detail(
    status: GameStatus,
    claimed_draw: Option<ClaimedDrawReason>,
) -> String {
    if let Some(claimed_draw_reason) = claimed_draw {
        return match claimed_draw_reason {
            ClaimedDrawReason::ThreefoldRepetition => {
                String::from("Threefold repetition was claimed from the in-match HUD.")
            }
            ClaimedDrawReason::FiftyMoveRule => {
                String::from("The fifty-move rule was claimed from the in-match HUD.")
            }
        };
    }

    match status {
        GameStatus::Ongoing { .. } => {
            String::from("The shell routes to results only after chess_core resolves the outcome.")
        }
        GameStatus::Finished(GameOutcome::Win {
            reason: WinReason::Checkmate,
            ..
        }) => String::from("Checkmate detected by chess_core."),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate)) => {
            String::from("Stalemate detected by chess_core.")
        }
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::FivefoldRepetition,
        ))) => String::from("Fivefold repetition detected by chess_core."),
        GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
            AutomaticDrawReason::SeventyFiveMoveRule,
        ))) => String::from("Seventy-five move rule detected by chess_core."),
    }
}
