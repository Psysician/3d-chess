use serde::{Deserialize, Serialize};

use crate::{Side, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl CastlingRights {
    #[must_use]
    pub const fn new(
        white_kingside: bool,
        white_queenside: bool,
        black_kingside: bool,
        black_queenside: bool,
    ) -> Self {
        Self {
            white_kingside,
            white_queenside,
            black_kingside,
            black_queenside,
        }
    }

    #[must_use]
    pub const fn standard() -> Self {
        Self::new(true, true, true, true)
    }

    #[must_use]
    pub const fn kingside(self, side: Side) -> bool {
        match side {
            Side::White => self.white_kingside,
            Side::Black => self.black_kingside,
        }
    }

    #[must_use]
    pub const fn queenside(self, side: Side) -> bool {
        match side {
            Side::White => self.white_queenside,
            Side::Black => self.black_queenside,
        }
    }

    pub fn revoke_kingside(&mut self, side: Side) {
        match side {
            Side::White => self.white_kingside = false,
            Side::Black => self.black_kingside = false,
        }
    }

    pub fn revoke_queenside(&mut self, side: Side) {
        match side {
            Side::White => self.white_queenside = false,
            Side::Black => self.black_queenside = false,
        }
    }

    pub fn revoke_side(&mut self, side: Side) {
        self.revoke_kingside(side);
        self.revoke_queenside(side);
    }

    pub fn revoke_rook_origin(&mut self, square: Square) {
        match (square.file(), square.rank()) {
            (0, 0) => self.white_queenside = false,
            (7, 0) => self.white_kingside = false,
            (0, 7) => self.black_queenside = false,
            (7, 7) => self.black_kingside = false,
            _ => {}
        }
    }

    #[must_use]
    pub fn to_fen(self) -> String {
        let mut fen = String::new();
        if self.white_kingside {
            fen.push('K');
        }
        if self.white_queenside {
            fen.push('Q');
        }
        if self.black_kingside {
            fen.push('k');
        }
        if self.black_queenside {
            fen.push('q');
        }

        if fen.is_empty() {
            String::from("-")
        } else {
            fen
        }
    }
}
