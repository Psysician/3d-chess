use chess_persistence::{RecoveryStartupPolicy, SavedSessionSummary};

use crate::plugins::menu::RecoveryBannerState;
use crate::plugins::save_load::SaveLoadState;

pub fn combine_persistence_errors(
    errors: impl IntoIterator<Item = Option<String>>,
) -> Option<String> {
    let messages = errors.into_iter().flatten().collect::<Vec<_>>();
    if messages.is_empty() {
        None
    } else {
        Some(messages.join(" "))
    }
}

pub fn manual_save_message(summary: &SavedSessionSummary) -> String {
    format!("Saved match as {}.", summary.label)
}

pub fn deleted_save_message(slot_id: &str) -> String {
    format!("Deleted save {slot_id}.")
}

pub fn recovery_banner_label(recovery: Option<&SavedSessionSummary>) -> Option<String> {
    recovery.map(|summary| summary.label.clone())
}

pub fn hide_recovery_banner(recovery_banner: &mut RecoveryBannerState) {
    recovery_banner.available = false;
    recovery_banner.dirty = false;
    recovery_banner.label = None;
}

pub fn sync_cached_recovery_visibility(
    save_state: &SaveLoadState,
    recovery_banner: &mut RecoveryBannerState,
) {
    let Some(summary) = save_state.recovery.as_ref() else {
        hide_recovery_banner(recovery_banner);
        return;
    };

    if save_state.settings.recovery_policy == RecoveryStartupPolicy::Ignore {
        hide_recovery_banner(recovery_banner);
        return;
    }

    recovery_banner.available = true;
    recovery_banner.dirty = false;
    recovery_banner.label = recovery_banner_label(Some(summary));
}

pub fn recovery_policy_status_copy(policy: RecoveryStartupPolicy) -> &'static str {
    match policy {
        RecoveryStartupPolicy::Resume => {
            "Resume automatically routes the stored interrupted session through MatchLoading."
        }
        RecoveryStartupPolicy::Ask => {
            "Ask keeps interrupted-session recovery visible without forcing a startup route."
        }
        RecoveryStartupPolicy::Ignore => {
            "Ignore hides interrupted-session affordances without deleting the stored snapshot."
        }
    }
}
