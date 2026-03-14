pub mod board;
pub mod game;
pub mod pieces;
pub mod square;

pub use board::BoardState;
pub use game::GameState;
pub use pieces::{Piece, PieceKind, Side};
pub use square::Square;

#[cfg(test)]
mod tests {
    use crate::Square;

    #[test]
    fn square_coordinates_are_bounded_to_the_board() {
        assert!(Square::new(0, 0).is_some());
        assert!(Square::new(7, 7).is_some());
        assert!(Square::new(8, 0).is_none());
        assert!(Square::new(0, 8).is_none());
    }
}
