use super::*;
use position::Position;
use pieces::{Piece, PieceColor, PieceKind};
use moves::{Move, MoveKind};

#[derive(Debug, Clone, Default)]
pub struct Board {
    piece_board: bitboards::PieceBoard,
    white_king_pos: Option<Position>,
    black_king_pos: Option<Position>,
    all_pieces: bitboards::BitBoard,
    all_white_pieces: bitboards::BitBoard,
    all_black_pieces: bitboards::BitBoard,
    white_pieces: bitboards::PieceWiseBitBoard,
    black_pieces: bitboards::PieceWiseBitBoard,
    white_in_check: std::sync::OnceLock<bool>,
    black_in_check: std::sync::OnceLock<bool>,
    side_to_move: PieceColor,
    ply_count: usize,
    game_state_history: Vec<GameState>,
    repeat_pos_history: Vec<zobrist::ZobristHash>,
    all_game_moves: Vec<Move>    
}

impl Board {
    pub const STANDARD_STARTING_BOARD: [Option<Piece>; 64] = [
        Some(Piece::WHITE_ROOK), Some(Piece::WHITE_KNIGHT), Some(Piece::WHITE_BISHOP), Some(Piece::WHITE_QUEEN), 
        Some(Piece::WHITE_KING), Some(Piece::WHITE_BISHOP), Some(Piece::WHITE_KNIGHT), Some(Piece::WHITE_ROOK),
        
        Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN),
        Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN), Some(Piece::WHITE_PAWN),

        None,        None,        None,        None,        None,        None,        None,        None,
        None,        None,        None,        None,        None,        None,        None,        None,
        None,        None,        None,        None,        None,        None,        None,        None,
        None,        None,        None,        None,        None,        None,        None,        None,

        Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN),
        Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN), Some(Piece::BLACK_PAWN),

        Some(Piece::BLACK_ROOK), Some(Piece::BLACK_KNIGHT), Some(Piece::BLACK_BISHOP), Some(Piece::BLACK_QUEEN), 
        Some(Piece::BLACK_KING), Some(Piece::BLACK_BISHOP), Some(Piece::BLACK_KNIGHT), Some(Piece::BLACK_ROOK),
    ];

    pub fn new() -> Self {
        let mut this = Self::default();
        this.reset();
        this
    }

    pub fn reset_to(&mut self, board: [Option<Piece>; 64]) {
        *self = Default::default();
        self.game_state_history.push(GameState::default());

        for (i, piece) in board.into_iter().enumerate() {
            let pos = Position::new((i >> 3) as u8, (i & 0x7) as u8);
            self.set(pos, piece);
        }
    }

    pub fn reset(&mut self) {
        self.reset_to(Self::STANDARD_STARTING_BOARD);
    }

    pub fn from_fen(fen: &str) -> anyhow::Result<Self> {
        let mut this = Self::default();
        let mut rank = 7u8;
        let mut file = 0u8;
        for c in fen.chars() {
            let pos = Position::new(rank, file);
            if let Some(piece) = Piece::from_char(c) {
                
                this.set(pos, Some(piece));
                file += 1;
            } else {
                match c {
                    '1' => file += 1,
                    '2' => file += 2,
                    '3' => file += 3,
                    '4' => file += 4,
                    '5' => file += 5,
                    '6' => file += 6,
                    '7' => file += 7,
                    '8' => file += 8,
                    '/' => {
                        rank -= 1;
                        file = 0;
                    },
                    c => anyhow::bail!("Unexpected character {c} in FEN string")
                }
            }
            if file >= 8 {
                file = 0;
                rank -= 1;
            }
        }

        Ok(this)
    }

    pub fn get(&self, pos: Position) -> Option<Piece> {
        self.piece_board.get(pos)
    }

    pub fn remove(&mut self, pos: Position) -> Option<Piece> {
        self.set(pos, None)
    }

    pub fn set(&mut self, pos: Position, piece: Option<Piece>) -> Option<Piece> {
        let old_piece = self.piece_board.get(pos);
        match old_piece.map(|piece| (piece.color, piece.kind)) {
            Some((PieceColor::White, kind)) => {
                self.all_pieces.clear(pos);
                self.all_white_pieces.clear(pos);
                self.white_pieces.clear(pos, kind);
                if kind == PieceKind::King {
                    self.white_king_pos.take();
                }
            },
            Some((PieceColor::Black, kind)) => {
                self.all_pieces.clear(pos);
                self.all_black_pieces.clear(pos);
                self.black_pieces.clear(pos, kind);
                if kind == PieceKind::King {
                    self.black_king_pos.take();
                }
            },
            None => {},
        }
        self.piece_board.set(pos, piece);
        match piece.map(|piece| (piece.color, piece.kind)) {
            Some((PieceColor::White, kind)) => {
                self.all_pieces.set(pos);
                self.all_white_pieces.set(pos);
                self.white_pieces.set(pos, kind);
                if kind == PieceKind::King {
                    self.white_king_pos.replace(pos);
                }
            },
            Some((PieceColor::Black, kind)) => {
                self.all_pieces.set(pos);
                self.all_black_pieces.set(pos);
                self.black_pieces.set(pos, kind);
                if kind == PieceKind::King {
                    self.black_king_pos.replace(pos);
                }
            },
            None => {},
        }
        old_piece
    }

    pub fn all_pieces(&self) -> bitboards::BitBoard {
        self.all_pieces
    }

    pub fn pieces(&self, color: PieceColor) -> bitboards::BitBoard {
        match color {
            PieceColor::White => self.all_white_pieces,
            PieceColor::Black => self.all_black_pieces,
        }
    }

    pub fn pieces_of(&self, color: PieceColor, kind: PieceKind) -> bitboards::BitBoard {
        match color {
            PieceColor::White => self.white_pieces.bitboard_of(kind),
            PieceColor::Black => self.black_pieces.bitboard_of(kind),
        }
    }

    pub fn pieces_of_color(&self, color: PieceColor) -> bitboards::PieceWiseBitBoard {
        match color {
            PieceColor::White => self.white_pieces,
            PieceColor::Black => self.black_pieces,
        }
    }


    pub fn king_pos(&self, color: PieceColor) -> Position {
        match color {
            PieceColor::White => self.white_king_pos.unwrap(),
            PieceColor::Black => self.black_king_pos.unwrap(),
        }
    }

    pub fn total_value(&self, color: PieceColor) -> i32 {
        let mut sum = 0;
        for pos in self.pieces(color).iter_positions() {
            if let Some(piece) = self.get(pos) {
                sum += piece.value();
            }
        }
        sum
    }

    pub fn side_to_move(&self) -> PieceColor {
        self.side_to_move
    }

    pub fn can_kingside_castle(&self, color: PieceColor) -> bool {
        self.current_game_state().castle_state.get(match color {
            PieceColor::White => CastleKind::WhiteKingside,
            PieceColor::Black => CastleKind::BlackKingside,
        })
    }

    pub fn can_queenside_castle(&self, color: PieceColor) -> bool {
        self.current_game_state().castle_state.get(match color {
            PieceColor::White => CastleKind::WhiteQueenside,
            PieceColor::Black => CastleKind::BlackQueenside,
        })
    }

    pub fn en_passant_file(&self) -> Option<u8> {
        self.current_game_state().en_passant_file
    }

    fn current_game_state(&self) -> &GameState {
        self.game_state_history.last().unwrap()
    }

    pub fn make_move(&mut self, move_to_make: Move, promotion: Option<PromotionKind>, in_search: bool) {
        let src = move_to_make.src();
        let dst = move_to_make.dst();
        let move_kind = move_to_make.kind();

        let piece = self.get(src).unwrap();
        let captured_piece = if move_kind == MoveKind::EnPassant {
            Some(Piece::new(PieceKind::Pawn, !self.side_to_move))
        } else {
            self.get(dst).and_then(|piece| if piece.color != self.side_to_move { Some(piece) } else { None })
        };

        let current_game_state = *self.current_game_state();

        let prev_castle_state = current_game_state.castle_state;
        let prev_en_passant_file = current_game_state.en_passant_file;
        let mut new_zobrist_hash = current_game_state.zobrist_hash;
        let mut new_castle_state = current_game_state.castle_state;
        let mut new_en_passant_file = None;

        self.remove(src);
        self.set(dst, Some(piece));

        if captured_piece.is_some() {
            let mut capture_pos = dst;
            if move_kind == MoveKind::EnPassant {
                capture_pos = Position::new(dst.rank().wrapping_add_signed(match self.side_to_move {
                    PieceColor::White =>  1i8,
                    PieceColor::Black => -1i8,
                }), dst.file());
                self.remove(capture_pos);
            }

            new_zobrist_hash.update_with_piece_pos(captured_piece, capture_pos);
        }

        if piece.kind == PieceKind::King {
            match self.side_to_move {
                PieceColor::White => {
                    new_castle_state.clear(CastleKind::WhiteKingside);
                    new_castle_state.clear(CastleKind::WhiteQueenside);
                },
                PieceColor::Black => {
                    new_castle_state.clear(CastleKind::BlackKingside);
                    new_castle_state.clear(CastleKind::BlackQueenside);
                },
            }

            if move_kind == MoveKind::Castle {
                let rook = Piece::new(PieceKind::Rook, self.side_to_move);
                let kingside = dst.file() >= 4;
                let rook_src = Position::new(dst.rank(), if kingside {
                    dst.file() + 1
                } else {
                    dst.file() - 2
                });
                let rook_dst = Position::new(dst.rank(), if kingside {
                    dst.file() - 1
                } else {
                    dst.file() + 1
                });
                self.remove(rook_src);
                self.set(rook_dst, Some(rook));

                new_zobrist_hash.update_with_piece_pos(Some(rook), rook_src);
                new_zobrist_hash.update_with_piece_pos(Some(rook), rook_dst);
            }
        }

        if move_kind == MoveKind::Promotion {
            let promoted_piece = Piece::new(match promotion.expect("Expected a promotion kind when promotion is specified") {
                moves::PromotionKind::Queen => PieceKind::Queen,
                moves::PromotionKind::Knight => PieceKind::Knight,
                moves::PromotionKind::Rook => PieceKind::Rook,
                moves::PromotionKind::Bishop => PieceKind::Bishop,
            }, self.side_to_move);

            self.set(dst, Some(promoted_piece));
        }

        if move_kind == MoveKind::PawnDoublePush {
            new_en_passant_file = Some(src.file());
            new_zobrist_hash.update_with_en_passant_file(new_en_passant_file);
        }

        {
            if dst == Position::H1 || src == Position::H1 {
                new_castle_state.clear(CastleKind::WhiteKingside);
            } else if dst == Position::A1 || src == Position::A1 {
                new_castle_state.clear(CastleKind::WhiteQueenside);
            }
            if dst == Position::H8 || src == Position::H8 {
                new_castle_state.clear(CastleKind::BlackKingside);
            } else if dst == Position::A8 || src == Position::A8 {
                new_castle_state.clear(CastleKind::BlackQueenside);
            }
        }

        new_zobrist_hash.update_with_side_to_move();
        new_zobrist_hash.update_with_piece_pos(Some(piece), src);
        new_zobrist_hash.update_with_piece_pos(self.get(dst), dst);
        new_zobrist_hash.update_with_en_passant_file(prev_en_passant_file);

        if new_castle_state != prev_castle_state {
            new_zobrist_hash.update_with_castle_state(prev_castle_state);
            new_zobrist_hash.update_with_castle_state(new_castle_state);
        }

        self.side_to_move = !self.side_to_move;
        self.ply_count += 1;

        let mut new_fifty_move_counter = current_game_state.fifty_move_counter + 1;

        if piece.kind == PieceKind::Pawn || captured_piece.is_some() {
            if !in_search {
                self.repeat_pos_history.clear();
            }
            new_fifty_move_counter = 0;
        }

        let new_state = GameState {
            captured_piece_kind: captured_piece.map(|piece| piece.kind),
            en_passant_file: new_en_passant_file,
            castle_state: new_castle_state,
            fifty_move_counter: new_fifty_move_counter,
            zobrist_hash: new_zobrist_hash,
        };
        self.game_state_history.push(new_state);
        
        match self.side_to_move {
            PieceColor::White => &mut self.white_in_check,
            PieceColor::Black => &mut self.black_in_check,
        }.take();

        if !in_search {
            self.repeat_pos_history.push(new_zobrist_hash);
            self.all_game_moves.push(move_to_make);
        }
    }

    pub fn unmake_move(&mut self, move_to_unmake: Move, in_search: bool) {
        self.side_to_move = !self.side_to_move;

        let current_game_state = *self.current_game_state();

        let src = move_to_unmake.src();
        let dst = move_to_unmake.dst();
        let move_kind = move_to_unmake.kind();

        let piece = if move_kind == MoveKind::Promotion {
            Piece::new(PieceKind::Pawn, self.side_to_move)
        } else {
            self.get(dst).unwrap()
        };
        let captured_piece = current_game_state.captured_piece_kind.map(|piece_kind| Piece::new(piece_kind, !self.side_to_move));

        self.remove(dst);
        self.set(src, Some(piece));

        if let Some(captured_piece) = captured_piece {
            let mut capture_pos = dst;
            if move_kind == MoveKind::EnPassant {
                capture_pos = Position::new(dst.rank().wrapping_add_signed(match self.side_to_move {
                    PieceColor::White =>  1i8,
                    PieceColor::Black => -1i8,
                }), dst.file());
            }
            self.set(capture_pos, Some(captured_piece));
        }

        if piece.kind == PieceKind::King {
            if move_kind == MoveKind::Castle {
                let rook = Piece::new(PieceKind::Rook, self.side_to_move);
                let kingside = dst.file() >= 4;
                let rook_src = Position::new(dst.rank(), if kingside {
                    dst.file() + 1
                } else {
                    dst.file() - 2
                });
                let rook_dst = Position::new(dst.rank(), if kingside {
                    dst.file() - 1
                } else {
                    dst.file() + 1
                });
                self.remove(rook_dst);
                self.set(rook_src, Some(rook));
            }
        }

        if !in_search {
            self.repeat_pos_history.pop();
            self.all_game_moves.pop();
        }

        self.game_state_history.pop();
        self.ply_count -= 1;
        match self.side_to_move {
            PieceColor::White => &mut self.white_in_check,
            PieceColor::Black => &mut self.black_in_check,
        }.take();
    }

    pub fn in_check(&self, color: PieceColor) -> bool {
        *match color {
            PieceColor::White => &self.white_in_check,
            PieceColor::Black => &self.black_in_check,
        }.get_or_init(|| self.calculate_checked_state(color))
    }

    fn calculate_checked_state(&self, color: PieceColor) -> bool {
        let king_pos = self.king_pos(color);
        let enemy_pieces = self.pieces_of_color(!color);
        let blockers = self.all_pieces();
        let enemy_ortho_sliders = self.pieces_of(!color, PieceKind::Rook) | self.pieces_of(!color, PieceKind::Queen);
        let enemy_diag_sliders = self.pieces_of(!color, PieceKind::Bishop) | self.pieces_of(!color, PieceKind::Queen);
        
        if !enemy_ortho_sliders.is_empty() {
            let ortho_attacks = magic::get_orthogonal_attacks(king_pos, blockers);
            if !(ortho_attacks & enemy_ortho_sliders).is_empty() {
                return true;
            }
        }

        if !enemy_diag_sliders.is_empty() {
            let diag_attacks = magic::get_diagonal_attacks(king_pos, blockers);
            if !(diag_attacks & enemy_diag_sliders).is_empty() {
                return true;
            }
        }

        if !enemy_pieces.knights.is_empty() {
            let knight_attacks = magic::get_knight_attacks(king_pos);
            if !(knight_attacks & enemy_pieces.knights).is_empty() {
                return true;
            }
        }

        if !enemy_pieces.pawns.is_empty() {
            let pawn_attacks = magic::get_pawn_attacks(king_pos, !color);
            if !(pawn_attacks & enemy_pieces.pawns).is_empty() {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastleKind {
    WhiteKingside,
    WhiteQueenside,
    BlackKingside,
    BlackQueenside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastleState(u8);

impl Default for CastleState {
    fn default() -> Self {
        Self(0xF)
    }
}

impl CastleState {
    pub fn new(white_kingside: bool, white_queenside: bool, black_kingside: bool, black_queenside: bool) -> Self {
        let mut this = Self::default();
        if !white_kingside {
            this.clear(CastleKind::WhiteKingside);
        }
        if !white_queenside {
            this.clear(CastleKind::WhiteQueenside);
        }
        if !black_kingside {
            this.clear(CastleKind::BlackKingside);
        }
        if !black_queenside {
            this.clear(CastleKind::BlackQueenside);
        }
        this
    }

    pub fn get(&self, kind: CastleKind) -> bool {
        self.0 & match kind {
            CastleKind::WhiteKingside =>  0b0001,
            CastleKind::WhiteQueenside => 0b0010,
            CastleKind::BlackKingside =>  0b0100,
            CastleKind::BlackQueenside => 0b1000,
        } != 0
    }

    pub fn clear(&mut self, kind: CastleKind) {
        self.0 &= !match kind {
            CastleKind::WhiteKingside =>  0b0001,
            CastleKind::WhiteQueenside => 0b0010,
            CastleKind::BlackKingside =>  0b0100,
            CastleKind::BlackQueenside => 0b1000,
        };
    }

    pub fn set(&mut self, kind: CastleKind) {
        self.0 |= match kind {
            CastleKind::WhiteKingside =>  0b0001,
            CastleKind::WhiteQueenside => 0b0010,
            CastleKind::BlackKingside =>  0b0100,
            CastleKind::BlackQueenside => 0b1000,
        };
    }
}

impl IntoPacked for CastleState {
    type Packed = u8;

    const MASK: Self::Packed = 0b1111;

    fn into_packed(self) -> Self::Packed {
        self.0
    }
}

impl FromPacked for CastleState {
    fn from_packed(packed: Self::Packed) -> Self {
        Self(packed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameState {
    pub captured_piece_kind: Option<PieceKind>,
    pub en_passant_file: Option<u8>,
    pub castle_state: CastleState,
    pub fifty_move_counter: u8,
    pub zobrist_hash: zobrist::ZobristHash,
}

impl GameState {

}