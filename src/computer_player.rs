use std::i32;

use rand::seq::{IndexedRandom, SliceRandom};

use crate::chess::*;



pub struct ComputerPlayer {
    color: PieceColor,
    move_generator: MoveGenerator,
    moves: Vec<Move>,

}

impl ComputerPlayer {
    pub fn new(color: PieceColor) -> Self {
        Self {
            color,
            move_generator: MoveGenerator::new(),
            moves: Vec::new()
        }
    }

    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    pub fn in_check(&self) -> bool {
        self.move_generator.in_check()
    }

    fn order_moves(moves: &mut [Move], board: &Board) {
        moves.sort_unstable_by_key(|current_move| {
            let mut score = 0;
            let piece = current_move.piece(board);
            if let Some(capture) = current_move.capture(board) {
                score = 10 * capture.value() - piece.value()
            }

            if current_move.kind() == MoveKind::Promotion {
                score += Piece::new(PieceKind::Queen, piece.color).value();
            }

            score
        });
    }
    
    fn evaluate(color: PieceColor, board: &Board) -> i32 {
        let our_value = board.total_value(color);
        let their_value = board.total_value(!color);
        our_value - their_value
    }
    
    fn search(&mut self, depth: usize, alpha: i32, beta: i32, color: PieceColor, board: &mut Board, best_move: &mut Option<Move>, iterations: &mut usize) -> i32 {
        if depth == 0 {
            return Self::evaluate(color, board);
        }
    
        let moves_start = self.moves.len();
    
        self.move_generator.generate_moves(board, &mut |mov| self.moves.push(mov), false);
    
        let moves_end = self.moves.len();
    
        Self::order_moves(&mut self.moves[moves_start..moves_end], &board);
    
        let ret = if moves_end == moves_start {
            // No available moves
            if board.in_check(color) {
                -i32::MAX// So that when it's negated it won't overflow 
            } else {
                0
            }
        } else {
            let mut best_evaluation = alpha;
            
            for i in moves_start..moves_end {
                let current_move = self.moves[i];
                board.make_move(current_move, Some(PromotionKind::Queen), true);
                let evaluation = -self.search(depth - 1, -beta, -alpha, !color, board, &mut None, iterations);
                board.unmake_move(current_move, true);
                *iterations += 1;
                if evaluation >= beta  {
                    best_evaluation = beta;
                    break;
                } else if evaluation >= best_evaluation {
                    best_evaluation = evaluation;
                    best_move.replace(current_move);
                    
                }
            }
            best_evaluation
        };
    
    
        self.moves.truncate(moves_start);
    
        ret
    }

    pub fn play(&mut self, board: &mut Board) -> Option<Move> {
        
        let mut best_move = None;
        let mut iterations = 0;
        let best_eval = self.search(4, -i32::MAX, i32::MAX, self.color, board, &mut best_move, &mut iterations);
        println!("Iterations searched: {}", iterations);
        if let Some(best_move) = best_move {
            println!("Evaluation: {}", best_eval);
            board.make_move(best_move, Some(PromotionKind::Queen), false);
            Some(best_move)
        } else {
            None
        }

        // self.move_generator.generate_moves(board, &mut |mov| self.moves.push(mov), false);
        // let chosen_move = self.moves.choose(&mut rand::rng()).copied();
        // if let Some(chosen_move) = chosen_move {
        //     board.make_move(chosen_move, Some(PromotionKind::Queen), false);
        // }
        // chosen_move
    }
}