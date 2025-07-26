use super::*;

struct ZobristData {
    pub pieces: [[u64; 16]; 64],
    pub castling_rights: [u64; 16],
    pub en_passant_file: [u64; 9],
    pub side_to_move: u64,
}

fn get_zobrist_data() -> &'static ZobristData {
    static ZOBRIST_DATA: std::sync::OnceLock<ZobristData> = std::sync::OnceLock::new();
    ZOBRIST_DATA.get_or_init(|| {
        use rand::*;
        let mut rng = rand::rng();

        ZobristData { 
            pieces: rng.random(), 
            castling_rights: rng.random(), 
            en_passant_file: rng.random(), 
            side_to_move: rng.random()
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZobristHash(u64);

impl ZobristHash {
    pub fn update_with_piece_pos(&mut self, piece: Option<pieces::Piece>, pos: position::Position) {
        self.0 ^= get_zobrist_data().pieces[pos.into_packed() as usize][piece.into_packed() as usize];
    }

    pub fn update_with_castle_state(&mut self, castling_rights: board::CastleState) {
        self.0 ^= get_zobrist_data().castling_rights[castling_rights.into_packed() as usize];
    }

    pub fn update_with_en_passant_file(&mut self, file: Option<u8>) {
        self.0 ^= get_zobrist_data().en_passant_file[if let Some(rank) = file { rank as usize } else { 8 }];
    }

    pub fn update_with_side_to_move(&mut self) {
        self.0 ^= get_zobrist_data().side_to_move;
    }
}