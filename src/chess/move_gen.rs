use std::collections::{HashMap, HashSet};

use crate::chess::position::Offset;

use super::*;
use bitboards::*;

pub const MAX_MOVES: usize = 218;

#[derive(Debug, Clone, Copy, Default)]
pub struct MoveGenerator {
    in_check: bool,
    in_double_check: bool,
    move_type_mask: BitBoard,
    check_ray_mask: BitBoard,
    pin_rays: BitBoard,
    enemy_attack_map_no_pawns: BitBoard,
    enemy_pawn_attack_map: BitBoard,
    enemy_sliding_attack_map: BitBoard,
    generate_quiet_moves: bool,
}

impl MoveGenerator {

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn in_check(&self) -> bool {
        self.in_check
    }

    fn color(&self, board: &Board) -> PieceColor {
        board.side_to_move()
    }

    fn reset(&mut self, board: &Board) {
        self.in_check = false;
        self.in_double_check = false;
        self.check_ray_mask = BitBoard::EMPTY;
        self.pin_rays = BitBoard::EMPTY;

        self.move_type_mask = if self.generate_quiet_moves { BitBoard::FULL } else { board.pieces(!self.color(board)) };
        
        self.calculate_attack_data(board);
    }

    fn calculate_attack_data(&mut self, board: &Board) {
        self.enemy_attack_map_no_pawns = BitBoard::EMPTY;
        // Update sliding piece attack map
        {
            self.enemy_sliding_attack_map = BitBoard::EMPTY;
            let blockers = {
                let mut pieces = board.all_pieces();
                pieces.clear(board.king_pos(self.color(board)));
                pieces
            };
            let ortho_sliders = board.pieces_of(!self.color(board), PieceKind::Rook) | board.pieces_of(!self.color(board), PieceKind::Queen);
            let diag_sliders = board.pieces_of(!self.color(board), PieceKind::Bishop) | board.pieces_of(!self.color(board), PieceKind::Queen);

            for pos in ortho_sliders.iter_positions() {
                self.enemy_sliding_attack_map |= magic::get_orthogonal_attacks(pos, blockers);
            }

            for pos in diag_sliders.iter_positions() {
                self.enemy_sliding_attack_map |= magic::get_diagonal_attacks(pos, blockers);
            }

            self.enemy_attack_map_no_pawns |= self.enemy_sliding_attack_map;
        }

        // Check for pins and sliding piece attacks 
        {

            let range = if !board.pieces_of(!self.color(board), PieceKind::Queen).is_empty() {
                0usize..8
            } else {
                (if !board.pieces_of(!self.color(board), PieceKind::Rook).is_empty() { 0usize } else { 4 })
                ..
                (if !board.pieces_of(!self.color(board), PieceKind::Bishop).is_empty() { 8usize } else { 4 })
            };

            for dir in Offset::DIRECTIONS[range].into_iter().copied() {
                let mut last_pos = board.king_pos(self.color(board));
                let mut ray = BitBoard::EMPTY;
                let mut has_pin = false;

                while let Some(pos) = last_pos.checked_offset(dir) {
                    last_pos = pos;
                    ray.set(pos);

                    if let Some(piece) = board.get(pos) {
                        if piece.color == self.color(board) {
                            if !has_pin {
                                has_pin = true;
                            } else {
                                break;
                            }
                        } else {
                            if piece.kind == PieceKind::Queen || (dir.is_diag() && piece.kind == PieceKind::Bishop) || (dir.is_ortho() && piece.kind == PieceKind::Rook) {
                                if has_pin {
                                    self.pin_rays |= ray;
                                } else {
                                    self.check_ray_mask |= ray;
                                    self.in_double_check = self.in_check;
                                    self.in_check = true;
                                }
                            }
                            break;
                        }
                    }
                }

                if self.in_double_check {
                    break;
                }
            }
        }

        // Check for knight attacks
        {
            let mut enemy_knight_attack_map = BitBoard::EMPTY;
            let knights = board.pieces_of(!self.color(board), PieceKind::Knight);

            for pos in knights.iter_positions() {
                let knight_attacks = magic::get_knight_attacks(pos);
                enemy_knight_attack_map |= knight_attacks;
                if knight_attacks.get(board.king_pos(self.color(board))) {
                    self.check_ray_mask.set(pos);
                    self.in_double_check = self.in_check;
                    self.in_check = true;
                }
            }
            self.enemy_attack_map_no_pawns |= enemy_knight_attack_map;
        }

        // Check for pawn attacks
        {
            let pawns = board.pieces_of(!self.color(board), PieceKind::Pawn);

            self.enemy_pawn_attack_map = magic::get_pawn_bitboard_attacks(pawns, !self.color(board));

            if self.enemy_pawn_attack_map.get(board.king_pos(self.color(board))) {
                self.check_ray_mask |= pawns & magic::get_pawn_attacks(board.king_pos(self.color(board)), self.color(board));
                self.in_double_check = self.in_check;
                self.in_check = true;
            }
        }

        // Check for king attacks
        {
            self.enemy_attack_map_no_pawns |= magic::get_king_moves(board.king_pos(!self.color(board)));
        }

        if !self.in_check {
            self.check_ray_mask = BitBoard::FULL;
        }
    }

    pub fn generate_moves<F: FnMut(Move)>(&mut self, board: &Board, push: &mut F, captures_only: bool) {
        self.generate_quiet_moves = !captures_only;
        self.reset(board);

        self.generate_king_moves(board, push);
        
        if !self.in_double_check {
            self.generate_sliding_moves(board, push);
            self.generate_knight_moves(board, push);
            self.generate_pawn_moves(board, push);
        }
    }

    fn generate_king_moves<F: FnMut(Move)>(&self, board: &Board, push: &mut F) {
        const WHITE_KINGSIDE_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::F1);
            mask.set(Position::G1);
            mask
        };
        const BLACK_KINGSIDE_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::F8);
            mask.set(Position::G8);
            mask
        };
        const WHITE_QUEENSIDE_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::D1);
            mask.set(Position::C1);
            mask
        };
        const BLACK_QUEENSIDE_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::D8);
            mask.set(Position::C8);
            mask
        };
        const WHITE_QUEENSIDE_BLOCK_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::D1);
            mask.set(Position::C1);
            mask.set(Position::B1);
            mask
        };
        const BLACK_QUEENSIDE_BLOCK_MASK: BitBoard = const {
            let mut mask = BitBoard::new();
            mask.set(Position::D8);
            mask.set(Position::C8);
            mask.set(Position::B8);
            mask
        };


        let legal_mask = !(self.enemy_attack_map_no_pawns | self.enemy_pawn_attack_map | board.pieces(self.color(board)));
        let king_moves = magic::get_king_moves(board.king_pos(self.color(board))) & legal_mask & self.move_type_mask;
        for dst in king_moves.iter_positions() {
            push(Move::new(board.king_pos(self.color(board)), dst, moves::MoveKind::Basic));
        }

        // Castling
        if !self.in_check && self.generate_quiet_moves {
            let castle_blockers = self.enemy_attack_map_no_pawns | self.enemy_pawn_attack_map | board.all_pieces();
            if board.can_kingside_castle(self.color(board)) {
                let (castle_mask, dst) = match self.color(board) {
                    PieceColor::White => (WHITE_KINGSIDE_MASK, Position::G1),
                    PieceColor::Black => (BLACK_KINGSIDE_MASK, Position::G8),
                };
                if (castle_mask & castle_blockers).is_empty() {
                    push(Move::new(board.king_pos(self.color(board)), dst, moves::MoveKind::Castle));
                }
            }
            if board.can_queenside_castle(self.color(board)) {
                let (castle_mask, castle_block_mask, dst) = match self.color(board) {
                    PieceColor::White => (WHITE_QUEENSIDE_MASK, WHITE_QUEENSIDE_BLOCK_MASK, Position::C1),
                    PieceColor::Black => (BLACK_QUEENSIDE_MASK, BLACK_QUEENSIDE_BLOCK_MASK, Position::C8),
                };
                if (castle_mask & castle_blockers).is_empty() && (castle_block_mask & board.all_pieces()).is_empty() {
                    push(Move::new(board.king_pos(self.color(board)), dst, moves::MoveKind::Castle));
                }
            }
        }
    }

    fn generate_sliding_moves<F: FnMut(Move)>(&self, board: &Board, push: &mut F) {
        let move_mask = !board.pieces(self.color(board)) & self.check_ray_mask & self.move_type_mask;

        let mut ortho_sliders = board.pieces_of(self.color(board), PieceKind::Rook) | board.pieces_of(self.color(board), PieceKind::Queen);
        let mut diag_sliders = board.pieces_of(self.color(board), PieceKind::Bishop) | board.pieces_of(self.color(board), PieceKind::Queen);

        if self.in_check {
            ortho_sliders &= !self.pin_rays;
            diag_sliders &= !self.pin_rays;
        }

        for src in ortho_sliders.iter_positions() {
            let mut move_positions = magic::get_orthogonal_attacks(src, board.all_pieces()) & move_mask;

            if self.pin_rays.get(src) {
                move_positions &= magic::get_dir_ray_mask(src, board.king_pos(self.color(board)))
            }

            for dst in move_positions.iter_positions() {
                push(Move::new(src, dst, moves::MoveKind::Basic));
            }
        }

        for src in diag_sliders.iter_positions() {
            let mut legal_moves = magic::get_diagonal_attacks(src, board.all_pieces()) & move_mask;

            if self.pin_rays.get(src) {
                legal_moves &= magic::get_dir_ray_mask(src, board.king_pos(self.color(board)))
            }

            for dst in legal_moves.iter_positions() {
                push(Move::new(src, dst, moves::MoveKind::Basic));
            }
        }
    }

    fn generate_knight_moves<F: FnMut(Move)>(&self, board: &Board, push: &mut F) {
        let move_mask = !board.pieces(self.color(board)) & self.check_ray_mask & self.move_type_mask;
        let knights = board.pieces_of(self.color(board), PieceKind::Knight) & !self.pin_rays;

        for src in knights.iter_positions() {
            let move_positions = magic::get_knight_attacks(src) & move_mask;

            for dst in move_positions.iter_positions() {
                push(Move::new(src, dst, moves::MoveKind::Basic));
            }
        }
    }

    fn generate_pawn_moves<F: FnMut(Move)>(&self, board: &Board, push: &mut F) {
        let king_pos = board.king_pos(self.color(board));

        let pawns = board.pieces_of(self.color(board), PieceKind::Pawn);

        let push_offset = match self.color(board) {
            PieceColor::White => Offset::N,
            PieceColor::Black => Offset::S,
        };

        let double_push_offset = push_offset * 2;

        let capture_w_offset = match self.color(board) {
            PieceColor::White => Offset::NW,
            PieceColor::Black => Offset::SW,
        };

        let capture_e_offset = match self.color(board) {
            PieceColor::White => Offset::NE,
            PieceColor::Black => Offset::SE,
        };

        let promotion_rank_mask = match self.color(board) {
            PieceColor::White => BitBoard::RANK_8,
            PieceColor::Black => BitBoard::RANK_1,
        };

        let double_push_rank_mask = match self.color(board) {
            PieceColor::White => BitBoard::RANK_4,
            PieceColor::Black => BitBoard::RANK_5,
        };

        let single_push = pawns.shift(push_offset) & !board.all_pieces();
        let double_push = single_push.shift(push_offset) & !board.all_pieces() & double_push_rank_mask & self.check_ray_mask;

        let single_push_promotions = single_push & promotion_rank_mask & self.check_ray_mask;
        let single_push_no_promotions = single_push & !promotion_rank_mask & self.check_ray_mask;

        let capture_w = (pawns & !BitBoard::FILE_A).shift(capture_w_offset) & board.pieces(!self.color(board));
        let capture_e = (pawns & !BitBoard::FILE_H).shift(capture_e_offset) & board.pieces(!self.color(board));

        let capture_w_promotions = capture_w & promotion_rank_mask & self.check_ray_mask;
        let capture_e_promotions = capture_e & promotion_rank_mask & self.check_ray_mask;

        let capture_w_no_promotions = capture_w & !promotion_rank_mask & self.check_ray_mask;
        let capture_e_no_promotions = capture_e & !promotion_rank_mask & self.check_ray_mask;

        if self.generate_quiet_moves {
            for dst in single_push_no_promotions.iter_positions() {
                let src = dst - push_offset;
                if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                    push(Move::new(src, dst, moves::MoveKind::Basic));
                }
            }

            for dst in double_push.iter_positions() {
                let src = dst - double_push_offset;
                if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                    push(Move::new(src, dst, moves::MoveKind::PawnDoublePush));
                }
            }
        }

        for dst in capture_w_no_promotions.iter_positions() {
            let src = dst - capture_w_offset;
            if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                push(Move::new(src, dst, moves::MoveKind::Basic));
            }
        }

        for dst in capture_e_no_promotions.iter_positions() {
            let src = dst - capture_e_offset;
            if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                push(Move::new(src, dst, moves::MoveKind::Basic));
            }
        }

        for dst in single_push_promotions.iter_positions() {
            let src = dst - push_offset;
            if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                self.generate_promotions(src, dst, push);
            }
        }

        for dst in capture_w_promotions.iter_positions() {
            let src = dst - capture_w_offset;
            if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                self.generate_promotions(src, dst, push);
            }
        }

        for dst in capture_e_promotions.iter_positions() {
            let src = dst - capture_e_offset;
            if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                self.generate_promotions(src, dst, push);
            }
        }

        if let Some(en_passant_file) = board.en_passant_file() {
            let en_passant_rank = match self.color(board) {
                PieceColor::White => 5,
                PieceColor::Black => 2,
            };

            let dst = Position::new(en_passant_rank, en_passant_file);
            let en_passant_capture = dst - push_offset;

            if self.check_ray_mask.get(en_passant_capture) {
                let pawns = pawns & magic::get_pawn_attacks(dst, !self.color(board));

                for src in pawns.iter_positions() {
                    if !self.pin_rays.get(src) || magic::get_align_mask(src, king_pos) == magic::get_align_mask(dst, king_pos) {
                        if !self.in_check_after_en_passant(src, dst, en_passant_capture, board) {
                            push(Move::new(src, dst, moves::MoveKind::EnPassant));
                        }
                    }
                }
            }
        }
    }

    fn generate_promotions<F: FnMut(Move)>(&self, src: Position, dst: Position, push: &mut F) {
        // push(Move::new(src, dst, moves::MoveKind::Promotion));
        push(Move::new(src, dst, moves::MoveKind::Promotion(PromotionKind::Queen)));
        if self.generate_quiet_moves {
            push(Move::new(src, dst, moves::MoveKind::Promotion(PromotionKind::Rook)));
            push(Move::new(src, dst, moves::MoveKind::Promotion(PromotionKind::Bishop)));
            push(Move::new(src, dst, moves::MoveKind::Promotion(PromotionKind::Knight)));
        }
    }

    fn in_check_after_en_passant(&self, src: Position, dst: Position, en_passant_capture: Position, board: &Board) -> bool {
        let ortho_attackers = board.pieces_of(!self.color(board), PieceKind::Rook) | board.pieces_of(!self.color(board), PieceKind::Queen);

        if !ortho_attackers.is_empty() {
            let masked_blockers = {
                let mut mask = board.all_pieces();
                mask.clear(src);
                mask.clear(dst);
                mask.clear(en_passant_capture);
                mask
            };
            let ortho_attacks = magic::get_orthogonal_attacks(board.king_pos(self.color(board)), masked_blockers);
            !(ortho_attackers & ortho_attackers).is_empty()
        } else { false }
    }
}

mod movegen_test {
    use crate::chess::{Board, Move, MoveGenerator, MoveKind, PromotionKind};

    #[test]
    fn move_count_test() {
        let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        let mut move_gen = MoveGenerator::new();
        let mut moves = Vec::new();
        println!("Move count: {}", count_moves(&mut board, &mut move_gen, &mut moves, 2, false, true));
    }

    fn count_moves(board: &mut Board, move_gen: &mut MoveGenerator, moves: &mut Vec<Move>, depth: usize, is_promotion: bool, is_outermost: bool) -> usize {
        if depth == 0 {
            if is_promotion {
                return 4;
            } else {
                return 1;
            }
        }

        let moves_start = moves.len();
    
        move_gen.generate_moves(board, &mut |mov| moves.push(mov), false);
    
        let moves_end = moves.len();

        moves[moves_start..moves_end].sort_unstable(); // In order to get the prints to line up with Stockfish's outputs 

        let mut count = 0usize;

        for i in moves_start..moves_end {
            let mov = moves[i];
            board.make_move(mov, true);
            let tmp = count_moves(board, move_gen, moves, depth-1, mov.is_promotion(), false);
            if is_outermost {
                println!("{mov}: {tmp}");
            }
            count += tmp;
            board.unmake_move(mov, true);
        }

        moves.truncate(moves_start);

        count
    }
}

struct MoveKey(u16);

#[derive(Debug, Clone)]
pub struct MoveTree {
    moves: Vec<Move>,
    src: BitBoard,
    dst: Box<[Option<BitBoard>; 64]>,
    kind: Box<[[Option<MoveKind>; 64]; 64]>
}

impl MoveTree {
    pub fn new() -> Self {
        Self {
            moves: Vec::with_capacity(MAX_MOVES),
            src: BitBoard::EMPTY,
            dst: Box::new([None; 64]),
            kind: Box::new([[None; 64]; 64]),
        }
    }

    pub fn push(&mut self, value: Move) {
        self.moves.push(value);
        self.src.set(value.src());
        self.dst[value.src().into_index() as usize].get_or_insert(BitBoard::EMPTY).set(value.dst());
        self.kind[value.src().into_index() as usize][value.dst().into_index() as usize].replace(value.kind());
    }

    pub fn clear(&mut self) {
        self.moves.clear();
        self.src = BitBoard::EMPTY;
        *self.dst = [None; 64];
        *self.kind = [[None; 64]; 64];
    }

    pub fn has_src(&self, src: Position) -> bool {
        self.src.get(src)
    }

    pub fn has_dst(&self, src: Position, dst: Position) -> bool {
        self.dst[src.into_index() as usize].is_some_and(|board| board.get(dst))
    }

    pub fn has_promotion(&self, src: Position, dst: Position) -> bool {
        self.kind[src.into_index() as usize][dst.into_index() as usize].is_some_and(|kind| kind.is_promotion())
    }

    pub fn get(&self, src: Position, dst: Position, promotion: Option<PromotionKind>) -> Option<Move> {
        if self.has_dst(src, dst) {
            let kind = self.kind[src.into_index() as usize][dst.into_index() as usize].unwrap();
            Some(Move::new(src, dst, if kind.is_promotion() {
                MoveKind::Promotion(promotion.expect("Promotion must be provided when move is a promotion."))
            } else { kind }))
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Move> {
        self.moves.iter()
    }

    pub fn src_positions(&self) -> impl Iterator<Item = Position> {
        self.src.iter_positions()
    }

    pub fn dst_positions(&self, src: Position) -> impl Iterator<Item = Position> {
        self.dst[src.into_index() as usize].map(|board| board.iter_positions()).into_iter().flatten()
    }

    pub fn move_count(&self) -> usize {
        self.moves.len()
    }
}