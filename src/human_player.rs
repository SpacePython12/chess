use std::collections::{HashMap, HashSet};

use crate::chess::*;

pub struct HumanPlayer {
    color: PieceColor,
    move_generator: MoveGenerator,
    move_tree: MoveTree,
    start_pos: Option<Position>,
    target_pos: Option<Position>,
    promotion: Option<PromotionKind>,
    in_turn: bool,
}

impl HumanPlayer {
    pub fn new(color: PieceColor) -> Self {
        Self {
            color,
            move_generator: MoveGenerator::new(),
            move_tree: MoveTree::new(),
            start_pos: None,
            target_pos: None,
            promotion: None,
            in_turn: false
        }
    }

    pub fn begin_turn(&mut self, board: &Board) {
        self.start_pos.take();
        self.target_pos.take();
        self.promotion.take();
        self.move_tree.clear();
        self.move_generator.generate_moves(board, &mut |mov| self.move_tree.push(mov), false);
        self.in_turn = true;
    }

    pub fn in_turn(&self) -> bool {
        self.in_turn
    }

    pub fn start_position(&self) -> Option<Position> {
        self.start_pos
    }

    pub fn move_count(&self) -> usize {
        self.move_tree.move_count()
    }

    pub fn in_check(&self) -> bool {
        self.move_generator.in_check()
    }

    pub fn cancel_move(&mut self) {
        self.start_pos.take();
        self.target_pos.take();
        self.promotion.take();
    }

    pub fn set_start_position(&mut self, start: Position) -> bool {
        if self.move_tree.has_src(start) {
            self.start_pos.replace(start);
            return true;
        }
        false
    }

    pub fn set_target_position(&mut self, target: Position) -> bool {
        if let Some(start) = self.start_pos {
            if self.move_tree.has_dst(start, target) {
                self.target_pos.replace(target);
                return true;
            }
        }
        false
    }

    pub fn can_move_to(&self, target: Position) -> bool {
        if let Some(start) = self.start_pos {
            if self.move_tree.has_dst(start, target) {
                return true;
            }
        }
        false
    }

    pub fn needs_promotion_choice(&self) -> bool {
        if let (Some(start_pos), Some(target_pos)) = (self.start_pos, self.target_pos) {
            if self.move_tree.has_promotion(start_pos, target_pos) {
                return true;
            }
        }
        false
    }

    pub fn promote(&mut self, promotion: PromotionKind) -> bool {
        if let (Some(start_pos), Some(target_pos)) = (self.start_pos, self.target_pos) {
            if self.move_tree.has_promotion(start_pos, target_pos) {
                self.promotion.replace(promotion);
                return true;
            }
        }
        false
    }

    pub fn finish_turn(&mut self, board: &mut Board) -> Option<Move> {
        if let (Some(start_pos), Some(target_pos), promotion) = (self.start_pos, self.target_pos, self.promotion) {
            if let Some(_move) = self.move_tree.get(start_pos, target_pos) {
                board.make_move(_move, promotion, false);
                self.in_turn = false;
                return Some(_move);
            }
        } 
        None
    }


}