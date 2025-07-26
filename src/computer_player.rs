use std::i32;

use rand::seq::{IndexedRandom, SliceRandom};

use crate::chess::*;



pub struct ComputerPlayer {
    color: PieceColor,
    move_generator: MoveGenerator,
    moves: Vec<Move>,
    current_move: Option<Move>,
}

impl ComputerPlayer {
    pub fn new(color: PieceColor) -> Self {
        Self {
            color,
            move_generator: MoveGenerator::new(),
            moves: Vec::new(),
            current_move: None
        }
    }

    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    pub fn in_check(&self) -> bool {
        self.move_generator.in_check()
    }

    fn order_moves(moves: &mut [Move], board: &Board) {
        moves.shuffle(&mut rand::rng());
        moves.sort_unstable_by_key(|current_move| {
            let mut score = 0;
            let piece = current_move.piece(board);
            if let Some(capture) = current_move.capture(board) {
                score = 10 * capture.value() - piece.value()
            }

            if current_move.is_promotion() {
                score += Piece::new(PieceKind::Queen, piece.color).value();
            }

            score
        });
    }
    
    fn evaluate(board: &Board) -> i32 {
        let our_value = board.total_value(board.side_to_move());
        let their_value = board.total_value(!board.side_to_move());
        our_value - their_value
    }
    
    fn search(&mut self, depth: usize, alpha: i32, beta: i32, board: &mut Board, best_move: &mut Option<Move>, iterations: &mut usize) -> i32 {
        if depth == 0 {
            return Self::evaluate(board);
        }
    
        let moves_start = self.moves.len();
    
        self.move_generator.generate_moves(board, &mut |mov| self.moves.push(mov), false);
    
        let moves_end = self.moves.len();
    
        Self::order_moves(&mut self.moves[moves_start..moves_end], &board);
    
        let ret = if moves_end == moves_start {
            // No available moves
            if board.in_check(board.side_to_move()) {
                -i32::MAX// So that when it's negated it won't overflow 
            } else {
                0
            }
        } else {
            let mut best_evaluation = alpha;
            
            for i in moves_start..moves_end {
                let current_move = self.moves[i];
                let evaluation = if current_move.capture(board).is_some_and(|piece| piece.kind == PieceKind::King) {
                    i32::MAX // It's a check!
                } else {
                    board.make_move(current_move, true);
                    let evaluation = -self.search(depth - 1, -beta, -alpha, board, &mut None, iterations);
                    board.unmake_move(current_move, true);
                    evaluation
                };
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

    pub fn begin_turn(&mut self, board: &mut Board) -> Option<Move> {
        
        let mut best_move = None;
        let mut iterations = 0;
        let best_eval = self.search(5, -i32::MAX, i32::MAX, board, &mut best_move, &mut iterations);
        println!("Iterations searched: {}", iterations);
        if let Some(best_move) = best_move {
            println!("Evaluation: {}", best_eval);
            self.current_move.replace(best_move);
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

    pub fn finish_turn(&mut self, board: &mut Board) {
        if let Some(current_move) = self.current_move.take() {
            board.make_move(current_move, false);
        }
    }
}