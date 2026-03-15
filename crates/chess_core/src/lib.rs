pub mod board;
pub mod castling;
pub mod game;
pub mod mv;
pub mod pieces;
pub mod square;
pub mod status;

pub use board::BoardState;
pub use castling::CastlingRights;
pub use game::{FenError, GameState};
pub use mv::{Move, MoveError};
pub use pieces::{Piece, PieceKind, Side};
pub use square::Square;
pub use status::{
    AutomaticDrawReason, DrawAvailability, DrawReason, GameOutcome, GameStatus, WinReason,
};
