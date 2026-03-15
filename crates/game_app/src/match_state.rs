use bevy::prelude::Resource;
use chess_core::{DrawAvailability, GameState, GameStatus, Move, MoveError, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimedDrawReason {
    ThreefoldRepetition,
    FiftyMoveRule,
}

// MatchSession is the sole Bevy-facing bridge to chess_core during M2 so presentation never becomes the rules authority.
// Pending promotion and claimable-draw state live here because they are interaction concerns layered on top of chess_core legality.
#[derive(Resource, Debug, Clone)]
pub struct MatchSession {
    pub game_state: GameState,
    pub selected_square: Option<Square>,
    pub pending_promotion_move: Option<Move>,
    pub last_move: Option<Move>,
    pub claimed_draw: Option<ClaimedDrawReason>,
}

impl MatchSession {
    #[must_use]
    pub fn start_local_match() -> Self {
        Self {
            game_state: GameState::starting_position(),
            selected_square: None,
            pending_promotion_move: None,
            last_move: None,
            claimed_draw: None,
        }
    }

    pub fn reset_for_local_match(&mut self) {
        *self = Self::start_local_match();
    }

    #[must_use]
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn replace_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
        self.last_move = None;
        self.claimed_draw = None;
        self.clear_interaction();
    }

    #[must_use]
    pub fn legal_targets_for_selected(&self) -> Vec<Square> {
        let Some(selected_square) = self.selected_square else {
            return Vec::new();
        };

        self.game_state
            .legal_moves()
            .into_iter()
            .filter(|candidate| candidate.from() == selected_square)
            .map(Move::to)
            .collect()
    }

    #[must_use]
    pub fn status(&self) -> GameStatus {
        self.game_state.status()
    }

    #[must_use]
    pub fn claimable_draw(&self) -> DrawAvailability {
        self.game_state.draw_availability()
    }

    pub fn clear_interaction(&mut self) {
        self.selected_square = None;
        self.pending_promotion_move = None;
    }

    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.game_state.piece_at(square)
    }

    pub fn apply_move(&mut self, candidate: Move) -> Result<(), MoveError> {
        let next_state = self.game_state.apply_move(candidate)?;
        self.game_state = next_state;
        self.last_move = Some(candidate);
        self.claimed_draw = None;
        self.clear_interaction();
        Ok(())
    }

    #[must_use]
    pub fn claimed_draw_reason(&self) -> Option<ClaimedDrawReason> {
        self.claimed_draw
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.claimed_draw.is_some() || self.status().is_finished()
    }

    pub fn claim_draw(&mut self) -> bool {
        let availability = self.claimable_draw();
        let reason = if availability.threefold_repetition {
            Some(ClaimedDrawReason::ThreefoldRepetition)
        } else if availability.fifty_move_rule {
            Some(ClaimedDrawReason::FiftyMoveRule)
        } else {
            None
        };

        let Some(reason) = reason else {
            return false;
        };

        self.claimed_draw = Some(reason);
        self.clear_interaction();
        true
    }
}

impl Default for MatchSession {
    fn default() -> Self {
        Self::start_local_match()
    }
}
