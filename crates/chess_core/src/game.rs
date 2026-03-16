//! FEN parsing helpers keep invalid-token branches distinct so rule coverage comes from deterministic domain logic instead of shell smoke tests. (ref: DL-003)
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    AutomaticDrawReason, BoardState, CastlingRights, DrawAvailability, DrawReason, GameOutcome,
    GameStatus, Move, MoveError, Piece, PieceKind, Side, Square, WinReason,
};

const BISHOP_DIRECTIONS: &[(i8, i8)] = &[(-1, -1), (-1, 1), (1, -1), (1, 1)];
const ROOK_DIRECTIONS: &[(i8, i8)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];
const QUEEN_DIRECTIONS: &[(i8, i8)] = &[
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
];
const KNIGHT_DELTAS: &[(i8, i8)] = &[
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];
const KING_DELTAS: &[(i8, i8)] = &[
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    InvalidFieldCount,
    InvalidBoard(String),
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEnPassantTarget,
    InvalidHalfmoveClock,
    InvalidFullmoveNumber,
    MissingKing(Side),
    MultipleKings(Side),
}

impl Display for FenError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFieldCount => formatter.write_str("FEN must contain exactly six fields"),
            Self::InvalidBoard(reason) => write!(formatter, "invalid board field: {reason}"),
            Self::InvalidSideToMove => formatter.write_str("invalid side-to-move token"),
            Self::InvalidCastlingRights => formatter.write_str("invalid castling-rights token"),
            Self::InvalidEnPassantTarget => formatter.write_str("invalid en-passant target"),
            Self::InvalidHalfmoveClock => formatter.write_str("invalid halfmove clock"),
            Self::InvalidFullmoveNumber => formatter.write_str("invalid fullmove number"),
            Self::MissingKing(side) => write!(formatter, "missing {side:?} king"),
            Self::MultipleKings(side) => write!(formatter, "multiple {side:?} kings"),
        }
    }
}

impl std::error::Error for FenError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    board: BoardState,
    side_to_move: Side,
    castling_rights: CastlingRights,
    en_passant_target: Option<Square>,
    halfmove_clock: u16,
    fullmove_number: u16,
    position_history: Vec<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::starting_position()
    }
}

impl GameState {
    #[must_use]
    pub fn starting_position() -> Self {
        let mut board = BoardState::empty();

        for file in 0_u8..8 {
            board.set_piece(
                Square::from_coords_unchecked(file, 1),
                Piece::new(Side::White, PieceKind::Pawn),
            );
            board.set_piece(
                Square::from_coords_unchecked(file, 6),
                Piece::new(Side::Black, PieceKind::Pawn),
            );
        }

        for (file, kind) in [
            (0, PieceKind::Rook),
            (1, PieceKind::Knight),
            (2, PieceKind::Bishop),
            (3, PieceKind::Queen),
            (4, PieceKind::King),
            (5, PieceKind::Bishop),
            (6, PieceKind::Knight),
            (7, PieceKind::Rook),
        ] {
            board.set_piece(
                Square::from_coords_unchecked(file, 0),
                Piece::new(Side::White, kind),
            );
            board.set_piece(
                Square::from_coords_unchecked(file, 7),
                Piece::new(Side::Black, kind),
            );
        }

        Self::from_parts(
            board,
            Side::White,
            CastlingRights::standard(),
            None,
            0,
            1,
            Vec::new(),
        )
    }

    #[must_use]
    pub fn board(&self) -> &BoardState {
        &self.board
    }

    #[must_use]
    pub const fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    #[must_use]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    #[must_use]
    pub const fn en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    #[must_use]
    pub const fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    #[must_use]
    pub const fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    #[must_use]
    pub fn position_history(&self) -> &[String] {
        &self.position_history
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board.piece_at(square)
    }

    #[must_use]
    pub fn status(&self) -> GameStatus {
        let legal_moves = self.legal_moves();
        let in_check = self.is_in_check(self.side_to_move);

        if legal_moves.is_empty() {
            if in_check {
                return GameStatus::Finished(GameOutcome::Win {
                    winner: self.side_to_move.opponent(),
                    reason: WinReason::Checkmate,
                });
            }

            return GameStatus::Finished(GameOutcome::Draw(DrawReason::Stalemate));
        }

        if self.halfmove_clock >= 150 {
            return GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
                AutomaticDrawReason::SeventyFiveMoveRule,
            )));
        }

        if self.current_position_repetition_count() >= 5 {
            return GameStatus::Finished(GameOutcome::Draw(DrawReason::Automatic(
                AutomaticDrawReason::FivefoldRepetition,
            )));
        }

        GameStatus::Ongoing {
            in_check,
            draw_available: self.draw_availability(),
        }
    }

    #[must_use]
    pub fn draw_availability(&self) -> DrawAvailability {
        DrawAvailability {
            threefold_repetition: self.current_position_repetition_count() >= 3,
            fifty_move_rule: self.halfmove_clock >= 100,
        }
    }

    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        self.legal_moves_for_side(self.side_to_move)
    }

    #[must_use]
    pub fn is_legal_move(&self, candidate: Move) -> bool {
        self.legal_moves().contains(&candidate)
    }

    pub fn apply_move(&self, candidate: Move) -> Result<Self, MoveError> {
        if self.status().is_finished() {
            return Err(MoveError::GameAlreadyFinished);
        }

        let piece = self
            .board
            .piece_at(candidate.from())
            .ok_or(MoveError::NoPieceAtSource)?;
        if piece.side != self.side_to_move {
            return Err(MoveError::WrongSideToMove);
        }
        if !self.is_legal_move(candidate) {
            return Err(MoveError::IllegalMove);
        }

        self.apply_move_unchecked(candidate)
    }

    #[must_use]
    pub fn to_fen(&self) -> String {
        let board = self.board_to_fen();
        let side_to_move = self.side_to_move.fen_token();
        let castling = self.castling_rights.to_fen();
        let en_passant = self
            .en_passant_target
            .map_or_else(|| String::from("-"), |square| square.to_string());

        format!(
            "{board} {side_to_move} {castling} {en_passant} {} {}",
            self.halfmove_clock, self.fullmove_number
        )
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let fields: Vec<_> = fen.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError::InvalidFieldCount);
        }

        let board = Self::parse_board(fields[0])?;
        Self::validate_king_count(&board, Side::White)?;
        Self::validate_king_count(&board, Side::Black)?;

        let side_to_move = Self::parse_side_to_move(fields[1])?;

        let castling_rights = Self::parse_castling_rights(fields[2])?;
        let en_passant_target = Self::parse_en_passant_target(fields[3])?;
        let (halfmove_clock, fullmove_number) = Self::parse_move_counters(fields[4], fields[5])?;

        Ok(Self::from_parts(
            board,
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
            Vec::new(),
        ))
    }

    fn parse_side_to_move(field: &str) -> Result<Side, FenError> {
        match field {
            "w" => Ok(Side::White),
            "b" => Ok(Side::Black),
            _ => Err(FenError::InvalidSideToMove),
        }
    }

    fn parse_move_counters(
        halfmove_clock: &str,
        fullmove_number: &str,
    ) -> Result<(u16, u16), FenError> {
        let halfmove_clock = halfmove_clock
            .parse::<u16>()
            .map_err(|_| FenError::InvalidHalfmoveClock)?;
        let fullmove_number = fullmove_number
            .parse::<u16>()
            .map_err(|_| FenError::InvalidFullmoveNumber)?;
        if fullmove_number == 0 {
            return Err(FenError::InvalidFullmoveNumber);
        }
        Ok((halfmove_clock, fullmove_number))
    }

    #[must_use]
    pub fn current_position_repetition_count(&self) -> usize {
        let current = self.current_position_key();
        self.position_history
            .iter()
            .filter(|entry| *entry == &current)
            .count()
    }

    #[must_use]
    pub fn is_in_check(&self, side: Side) -> bool {
        self.board
            .king_square(side)
            .is_some_and(|king_square| self.is_square_attacked(king_square, side.opponent()))
    }

    fn from_parts(
        board: BoardState,
        side_to_move: Side,
        castling_rights: CastlingRights,
        en_passant_target: Option<Square>,
        halfmove_clock: u16,
        fullmove_number: u16,
        mut position_history: Vec<String>,
    ) -> Self {
        let mut state = Self {
            board,
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
            position_history: Vec::new(),
        };

        if position_history.is_empty() {
            position_history.push(state.current_position_key());
        }

        state.position_history = position_history;
        state
    }

    fn legal_moves_for_side(&self, side: Side) -> Vec<Move> {
        let mut legal_moves = Vec::new();

        for (square, piece) in self.board.iter_side(side) {
            let pseudo_moves = self.pseudo_legal_moves_from(square, piece);
            for candidate in pseudo_moves {
                if let Ok(next_state) = self.apply_move_unchecked(candidate)
                    && !next_state.is_in_check(side)
                {
                    legal_moves.push(candidate);
                }
            }
        }

        legal_moves
    }

    fn pseudo_legal_moves_from(&self, square: Square, piece: Piece) -> Vec<Move> {
        match piece.kind {
            PieceKind::Pawn => self.generate_pawn_moves(square, piece.side),
            PieceKind::Knight => self.generate_jump_moves(square, piece.side, KNIGHT_DELTAS),
            PieceKind::Bishop => self.generate_sliding_moves(square, piece.side, BISHOP_DIRECTIONS),
            PieceKind::Rook => self.generate_sliding_moves(square, piece.side, ROOK_DIRECTIONS),
            PieceKind::Queen => self.generate_sliding_moves(square, piece.side, QUEEN_DIRECTIONS),
            PieceKind::King => self.generate_king_moves(square, piece.side),
        }
    }

    fn generate_pawn_moves(&self, square: Square, side: Side) -> Vec<Move> {
        let mut moves = Vec::new();
        let forward = side.pawn_forward();

        if let Some(forward_square) = square.offset(0, forward)
            && self.board.piece_at(forward_square).is_none()
        {
            self.push_pawn_moves(square, forward_square, side, &mut moves);

            if square.rank() == side.pawn_start_rank()
                && let Some(double_square) = square.offset(0, forward * 2)
                && self.board.piece_at(double_square).is_none()
            {
                moves.push(Move::new(square, double_square));
            }
        }

        for file_delta in [-1, 1] {
            if let Some(capture_square) = square.offset(file_delta, forward) {
                if let Some(target_piece) = self.board.piece_at(capture_square) {
                    if target_piece.side != side {
                        self.push_pawn_moves(square, capture_square, side, &mut moves);
                    }
                } else if self.en_passant_target == Some(capture_square) {
                    let captured_square =
                        Square::from_coords_unchecked(capture_square.file(), square.rank());
                    if self.board.piece_at(captured_square)
                        == Some(Piece::new(side.opponent(), PieceKind::Pawn))
                    {
                        moves.push(Move::new(square, capture_square));
                    }
                }
            }
        }

        moves
    }

    fn push_pawn_moves(&self, from: Square, to: Square, side: Side, moves: &mut Vec<Move>) {
        if to.rank() == side.promotion_rank() {
            for promotion in [
                PieceKind::Queen,
                PieceKind::Rook,
                PieceKind::Bishop,
                PieceKind::Knight,
            ] {
                moves.push(Move::with_promotion(from, to, promotion));
            }
        } else {
            moves.push(Move::new(from, to));
        }
    }

    fn generate_jump_moves(&self, square: Square, side: Side, deltas: &[(i8, i8)]) -> Vec<Move> {
        let mut moves = Vec::new();

        for &(file_delta, rank_delta) in deltas {
            if let Some(target) = square.offset(file_delta, rank_delta) {
                match self.board.piece_at(target) {
                    Some(piece) if piece.side == side => {}
                    _ => moves.push(Move::new(square, target)),
                }
            }
        }

        moves
    }

    fn generate_sliding_moves(
        &self,
        square: Square,
        side: Side,
        directions: &[(i8, i8)],
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        for &(file_step, rank_step) in directions {
            let mut cursor = square;
            while let Some(next) = cursor.offset(file_step, rank_step) {
                cursor = next;
                match self.board.piece_at(next) {
                    Some(piece) if piece.side == side => break,
                    Some(_) => {
                        moves.push(Move::new(square, next));
                        break;
                    }
                    None => moves.push(Move::new(square, next)),
                }
            }
        }

        moves
    }

    fn generate_king_moves(&self, square: Square, side: Side) -> Vec<Move> {
        let mut moves = self.generate_jump_moves(square, side, KING_DELTAS);
        let home_rank = side.home_rank();
        let opponent = side.opponent();

        if !self.is_in_check(side) {
            let kingside_path = [
                Square::from_coords_unchecked(5, home_rank),
                Square::from_coords_unchecked(6, home_rank),
            ];
            let queenside_path = [
                Square::from_coords_unchecked(3, home_rank),
                Square::from_coords_unchecked(2, home_rank),
            ];

            if self.castling_rights.kingside(side)
                && self
                    .board
                    .piece_at(Square::from_coords_unchecked(7, home_rank))
                    == Some(Piece::new(side, PieceKind::Rook))
                && kingside_path
                    .iter()
                    .all(|target| self.board.piece_at(*target).is_none())
                && kingside_path
                    .iter()
                    .all(|target| !self.is_square_attacked(*target, opponent))
            {
                moves.push(Move::new(
                    square,
                    Square::from_coords_unchecked(6, home_rank),
                ));
            }

            if self.castling_rights.queenside(side)
                && self
                    .board
                    .piece_at(Square::from_coords_unchecked(0, home_rank))
                    == Some(Piece::new(side, PieceKind::Rook))
                && [
                    Square::from_coords_unchecked(1, home_rank),
                    Square::from_coords_unchecked(2, home_rank),
                    Square::from_coords_unchecked(3, home_rank),
                ]
                .iter()
                .all(|target| self.board.piece_at(*target).is_none())
                && queenside_path
                    .iter()
                    .all(|target| !self.is_square_attacked(*target, opponent))
            {
                moves.push(Move::new(
                    square,
                    Square::from_coords_unchecked(2, home_rank),
                ));
            }
        }

        moves
    }

    fn apply_move_unchecked(&self, candidate: Move) -> Result<Self, MoveError> {
        let mut next = self.clone();
        let from = candidate.from();
        let to = candidate.to();
        let piece = next
            .board
            .remove_piece(from)
            .ok_or(MoveError::NoPieceAtSource)?;

        if piece.side != self.side_to_move {
            return Err(MoveError::WrongSideToMove);
        }

        if let Some(occupant) = next.board.piece_at(to)
            && occupant.side == piece.side
        {
            return Err(MoveError::IllegalMove);
        }

        let mut is_capture = false;
        let mut new_en_passant_target = None;

        if piece.kind == PieceKind::Pawn {
            let file_delta = from.file().abs_diff(to.file());

            if file_delta == 1 && next.board.piece_at(to).is_none() {
                if self.en_passant_target != Some(to) {
                    return Err(MoveError::IllegalMove);
                }

                let captured_square = Square::from_coords_unchecked(to.file(), from.rank());
                let captured_piece = next.board.remove_piece(captured_square);
                if captured_piece != Some(Piece::new(piece.side.opponent(), PieceKind::Pawn)) {
                    return Err(MoveError::IllegalMove);
                }
                is_capture = true;
            } else if next.board.remove_piece(to).is_some() {
                is_capture = true;
            }

            let promotion_rank = piece.side.promotion_rank();
            if to.rank() == promotion_rank {
                let promotion = candidate
                    .promotion()
                    .ok_or(MoveError::MissingPromotionChoice)?;
                if !promotion.is_valid_promotion() {
                    return Err(MoveError::InvalidPromotionChoice);
                }
                next.board.set_piece(to, Piece::new(piece.side, promotion));
            } else {
                if candidate.promotion().is_some() {
                    return Err(MoveError::InvalidPromotionChoice);
                }
                next.board.set_piece(to, piece);
            }

            if from.rank().abs_diff(to.rank()) == 2 {
                let passed_rank = (from.rank() + to.rank()) / 2;
                new_en_passant_target =
                    Some(Square::from_coords_unchecked(from.file(), passed_rank));
            }
        } else {
            if candidate.promotion().is_some() {
                return Err(MoveError::InvalidPromotionChoice);
            }
            if next.board.remove_piece(to).is_some() {
                is_capture = true;
            }
            next.board.set_piece(to, piece);
        }

        if piece.kind == PieceKind::King {
            next.castling_rights.revoke_side(piece.side);
            if from.file().abs_diff(to.file()) == 2 {
                let home_rank = piece.side.home_rank();
                if to.file() == 6 {
                    let rook_from = Square::from_coords_unchecked(7, home_rank);
                    let rook_to = Square::from_coords_unchecked(5, home_rank);
                    let rook = next.board.remove_piece(rook_from);
                    if rook != Some(Piece::new(piece.side, PieceKind::Rook)) {
                        return Err(MoveError::IllegalMove);
                    }
                    next.board
                        .set_piece(rook_to, Piece::new(piece.side, PieceKind::Rook));
                } else if to.file() == 2 {
                    let rook_from = Square::from_coords_unchecked(0, home_rank);
                    let rook_to = Square::from_coords_unchecked(3, home_rank);
                    let rook = next.board.remove_piece(rook_from);
                    if rook != Some(Piece::new(piece.side, PieceKind::Rook)) {
                        return Err(MoveError::IllegalMove);
                    }
                    next.board
                        .set_piece(rook_to, Piece::new(piece.side, PieceKind::Rook));
                } else {
                    return Err(MoveError::IllegalMove);
                }
            }
        }

        if piece.kind == PieceKind::Rook {
            next.castling_rights.revoke_rook_origin(from);
        }

        if is_capture {
            next.castling_rights.revoke_rook_origin(to);
        }

        next.en_passant_target = new_en_passant_target;
        next.halfmove_clock = if piece.kind == PieceKind::Pawn || is_capture {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };
        next.fullmove_number = if piece.side == Side::Black {
            self.fullmove_number.saturating_add(1)
        } else {
            self.fullmove_number
        };
        next.side_to_move = piece.side.opponent();
        next.position_history = self.position_history.clone();
        next.position_history.push(next.current_position_key());

        Ok(next)
    }

    fn is_square_attacked(&self, square: Square, by_side: Side) -> bool {
        let pawn_source_rank_delta = -by_side.pawn_forward();
        for file_delta in [-1, 1] {
            if let Some(pawn_square) = square.offset(file_delta, pawn_source_rank_delta)
                && self.board.piece_at(pawn_square) == Some(Piece::new(by_side, PieceKind::Pawn))
            {
                return true;
            }
        }

        for &(file_delta, rank_delta) in KNIGHT_DELTAS {
            if let Some(knight_square) = square.offset(file_delta, rank_delta)
                && self.board.piece_at(knight_square)
                    == Some(Piece::new(by_side, PieceKind::Knight))
            {
                return true;
            }
        }

        for &(file_delta, rank_delta) in KING_DELTAS {
            if let Some(king_square) = square.offset(file_delta, rank_delta)
                && self.board.piece_at(king_square) == Some(Piece::new(by_side, PieceKind::King))
            {
                return true;
            }
        }

        self.is_attacked_by_slider(
            square,
            by_side,
            BISHOP_DIRECTIONS,
            &[PieceKind::Bishop, PieceKind::Queen],
        ) || self.is_attacked_by_slider(
            square,
            by_side,
            ROOK_DIRECTIONS,
            &[PieceKind::Rook, PieceKind::Queen],
        )
    }

    fn is_attacked_by_slider(
        &self,
        square: Square,
        by_side: Side,
        directions: &[(i8, i8)],
        attackers: &[PieceKind],
    ) -> bool {
        for &(file_step, rank_step) in directions {
            let mut cursor = square;
            while let Some(next) = cursor.offset(file_step, rank_step) {
                cursor = next;
                if let Some(piece) = self.board.piece_at(next) {
                    if piece.side == by_side && attackers.contains(&piece.kind) {
                        return true;
                    }
                    break;
                }
            }
        }

        false
    }

    fn board_to_fen(&self) -> String {
        let mut ranks = Vec::new();

        for rank in (0_u8..8).rev() {
            let mut fen_rank = String::new();
            let mut empty_run = 0_u8;

            for file in 0_u8..8 {
                let square = Square::from_coords_unchecked(file, rank);
                if let Some(piece) = self.board.piece_at(square) {
                    if empty_run > 0 {
                        fen_rank.push(char::from(b'0' + empty_run));
                        empty_run = 0;
                    }
                    fen_rank.push(piece.fen_char());
                } else {
                    empty_run += 1;
                }
            }

            if empty_run > 0 {
                fen_rank.push(char::from(b'0' + empty_run));
            }

            ranks.push(fen_rank);
        }

        ranks.join("/")
    }

    fn current_position_key(&self) -> String {
        let en_passant = self
            .normalized_en_passant_target()
            .map_or_else(|| String::from("-"), |square| square.to_string());

        format!(
            "{} {} {} {}",
            self.board_to_fen(),
            self.side_to_move.fen_token(),
            self.castling_rights.to_fen(),
            en_passant
        )
    }

    fn normalized_en_passant_target(&self) -> Option<Square> {
        let target = self.en_passant_target?;
        let side = self.side_to_move;
        let capture_rank_delta = -side.pawn_forward();

        for file_delta in [-1, 1] {
            if let Some(pawn_square) = target.offset(file_delta, capture_rank_delta)
                && self.board.piece_at(pawn_square) == Some(Piece::new(side, PieceKind::Pawn))
            {
                let captured_square =
                    Square::from_coords_unchecked(target.file(), pawn_square.rank());
                if self.board.piece_at(captured_square)
                    == Some(Piece::new(side.opponent(), PieceKind::Pawn))
                {
                    return Some(target);
                }
            }
        }

        None
    }

    fn parse_board(board_field: &str) -> Result<BoardState, FenError> {
        let ranks: Vec<_> = board_field.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenError::InvalidBoard(String::from("expected 8 ranks")));
        }

        let mut board = BoardState::empty();
        for (index, rank_field) in ranks.iter().enumerate() {
            let rank = 7_u8
                .checked_sub(u8::try_from(index).map_err(|_| {
                    FenError::InvalidBoard(String::from("rank index out of bounds"))
                })?)
                .ok_or_else(|| FenError::InvalidBoard(String::from("rank index out of bounds")))?;
            let mut file = 0_u8;

            for token in rank_field.chars() {
                if token.is_ascii_digit() {
                    let skip = token
                        .to_digit(10)
                        .and_then(|value| u8::try_from(value).ok())
                        .ok_or_else(|| {
                            FenError::InvalidBoard(String::from("invalid empty-square digit"))
                        })?;
                    if !(1..=8).contains(&skip) {
                        return Err(FenError::InvalidBoard(String::from(
                            "empty-square digit must be between 1 and 8",
                        )));
                    }
                    file = file.saturating_add(skip);
                } else {
                    let piece = Piece::from_fen(token).ok_or_else(|| {
                        FenError::InvalidBoard(format!("invalid piece token '{token}'"))
                    })?;
                    if file >= 8 {
                        return Err(FenError::InvalidBoard(String::from(
                            "too many files in rank",
                        )));
                    }
                    board.set_piece(Square::from_coords_unchecked(file, rank), piece);
                    file = file.saturating_add(1);
                }
            }

            if file != 8 {
                return Err(FenError::InvalidBoard(String::from(
                    "rank does not sum to 8 files",
                )));
            }
        }

        Ok(board)
    }

    fn validate_king_count(board: &BoardState, side: Side) -> Result<(), FenError> {
        let king_count = board
            .iter_side(side)
            .filter(|(_, piece)| piece.kind == PieceKind::King)
            .count();

        match king_count {
            1 => Ok(()),
            0 => Err(FenError::MissingKing(side)),
            _ => Err(FenError::MultipleKings(side)),
        }
    }

    fn parse_castling_rights(field: &str) -> Result<CastlingRights, FenError> {
        if field == "-" {
            return Ok(CastlingRights::default());
        }

        let mut rights = CastlingRights::default();
        for token in field.chars() {
            match token {
                'K' => {
                    rights = CastlingRights::new(
                        true,
                        rights.queenside(Side::White),
                        rights.kingside(Side::Black),
                        rights.queenside(Side::Black),
                    )
                }
                'Q' => {
                    rights = CastlingRights::new(
                        rights.kingside(Side::White),
                        true,
                        rights.kingside(Side::Black),
                        rights.queenside(Side::Black),
                    )
                }
                'k' => {
                    rights = CastlingRights::new(
                        rights.kingside(Side::White),
                        rights.queenside(Side::White),
                        true,
                        rights.queenside(Side::Black),
                    )
                }
                'q' => {
                    rights = CastlingRights::new(
                        rights.kingside(Side::White),
                        rights.queenside(Side::White),
                        rights.kingside(Side::Black),
                        true,
                    )
                }
                _ => return Err(FenError::InvalidCastlingRights),
            }
        }

        Ok(rights)
    }

    fn parse_en_passant_target(field: &str) -> Result<Option<Square>, FenError> {
        if field == "-" {
            return Ok(None);
        }

        let square = Square::from_algebraic(field).ok_or(FenError::InvalidEnPassantTarget)?;
        if !matches!(square.rank(), 2 | 5) {
            return Err(FenError::InvalidEnPassantTarget);
        }

        Ok(Some(square))
    }
}
