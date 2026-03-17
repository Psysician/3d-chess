// Semantic automation dispatch lives in a dedicated plugin so commands flow
// through the same shell and match handlers instead of a parallel path.
// (refs: DL-001, DL-003)

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::app::AppScreenState;
use crate::automation::{
    capture_snapshot, AutomationCommand, AutomationError, AutomationHarness,
    AutomationResult, AutomationSnapshot,
};
use crate::match_state::MatchSession;

use super::app_shell::{
    handle_confirmation_action, handle_navigation_action, handle_save_slot_action,
    handle_settings_action,
};
use super::input::apply_match_action;
use super::menu::{MenuAction, ShellMenuState};
use super::save_load::{SaveLoadRequest, SaveLoadState};

#[derive(Resource, Default)]
struct AutomationCommandQueue(VecDeque<AutomationCommand>);

#[derive(Resource, Debug, Clone, Default)]
pub struct AutomationSnapshotResource(pub AutomationSnapshot);

#[derive(Resource, Debug, Clone, Default)]
struct AutomationLastError(Option<AutomationError>);

pub struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutomationCommandQueue>()
            .init_resource::<AutomationSnapshotResource>()
            .init_resource::<AutomationLastError>()
            .add_systems(
                Update,
                (dispatch_automation_commands, refresh_automation_snapshot).chain(),
            );
    }
}

fn dispatch_automation_commands(
    mut queue: ResMut<AutomationCommandQueue>,
    mut last_error: ResMut<AutomationLastError>,
    state: Res<State<AppScreenState>>,
    menu_state: Res<ShellMenuState>,
    save_state: Res<SaveLoadState>,
    mut menu_actions: MessageWriter<MenuAction>,
    mut save_requests: MessageWriter<SaveLoadRequest>,
    mut match_session: ResMut<MatchSession>,
) {
    if queue.0.is_empty() {
        return;
    }
    last_error.0 = None;
    while let Some(command) = queue.0.pop_front() {
        let result = match command {
            AutomationCommand::Navigation(action) => {
                handle_navigation_action(
                    action,
                    *state.get(),
                    menu_state.as_ref(),
                    save_state.as_ref(),
                    &mut menu_actions,
                    &mut save_requests,
                );
                Ok(())
            }
            AutomationCommand::Save(action) => handle_save_slot_action(
                &action,
                menu_state.as_ref(),
                save_state.as_ref(),
                &mut menu_actions,
                &mut save_requests,
                match_session.as_ref(),
            ),
            AutomationCommand::Settings(action) => {
                handle_settings_action(&action, &mut save_requests);
                Ok(())
            }
            AutomationCommand::Match(action) => {
                apply_match_action(match_session.as_mut(), &action)
            }
            AutomationCommand::Confirm(kind) => handle_confirmation_action(
                kind,
                menu_state.as_ref(),
                save_state.as_ref(),
                &mut menu_actions,
                &mut save_requests,
            ),
            AutomationCommand::Snapshot | AutomationCommand::Step { .. } => Ok(()),
        };

        if let Err(error) = result {
            last_error.0 = Some(error);
            break;
        }
    }
}

fn refresh_automation_snapshot(world: &mut World) {
    let snapshot = capture_snapshot(world);
    world.resource_mut::<AutomationSnapshotResource>().0 = snapshot;
}

impl AutomationHarness {
    #[must_use]
    pub fn with_semantic_automation(mut self) -> Self {
        self.ensure_semantic_automation();
        self
    }

    pub fn try_submit(&mut self, command: AutomationCommand) -> AutomationResult<AutomationSnapshot> {
        self.ensure_semantic_automation();
        match command {
            AutomationCommand::Snapshot => {}
            AutomationCommand::Step { frames } => {
                if frames == 0 {
                    return Err(AutomationError::InvalidStepCount(frames));
                }
                for _ in 0..frames {
                    self.app.update();
                }
            }
            command => {
                self.app.world_mut().resource_mut::<AutomationCommandQueue>().0.push_back(command);
                // Frame 1: dispatch_automation_commands runs and writes MenuAction / SaveLoadRequest messages.
                // Frame 2: downstream systems (save_load, menu routing) observe those messages and apply state changes.
                self.app.update();
                self.app.update();
                if let Some(error) = self.app.world_mut().resource_mut::<AutomationLastError>().0.take() {
                    return Err(error);
                }
            }
        }

        Ok(self.snapshot())
    }

    fn ensure_semantic_automation(&mut self) {
        if self.app.world().contains_resource::<AutomationCommandQueue>() {
            return;
        }

        self.app.add_plugins(AutomationPlugin);
        let snapshot = capture_snapshot(self.app.world());
        self.app.world_mut().resource_mut::<AutomationSnapshotResource>().0 = snapshot;
    }
}
