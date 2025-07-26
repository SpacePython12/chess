use std::ops::{Shl, Shr};

use super::{position::Offset, *};
use position::Position;
use pieces::{Piece, PieceColor, PieceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PieceBoard([u32; 8]);

impl From<[Option<Piece>; 64]> for PieceBoard {
    fn from(value: [Option<Piece>; 64]) -> Self {
        let mut this = Self::default();
        for (i, piece) in value.into_iter().enumerate() {
            let pos = Position::new((i >> 3) as u8, (i & 0x7) as u8);
            this.set(pos, piece);
        }
        this
    }
}

impl PieceBoard {
    #[inline]
    fn get_nybble(&self, pos: Position) -> u8 {
        ((self.0[pos.rank() as usize] >> (pos.file() << 2)) & 0xF) as u8
    }

    #[inline]
    fn set_nybble(&mut self, pos: Position, nybble: u8) {
        self.0[pos.rank() as usize] &= !(0xF << (pos.file() << 2));
        self.0[pos.rank() as usize] |= ((nybble & 0xF) as u32) << (pos.file() << 2);
    }

    pub fn get(&self, pos: Position) -> Option<Piece> {
        Option::<Piece>::from_packed(self.get_nybble(pos))
    }

    pub fn set(&mut self, pos: Position, piece: Option<Piece>) {
        self.set_nybble(pos, piece.into_packed());
    }

    pub fn is_empty(&self, pos: Position) -> bool {
        self.get_nybble(pos) & 0x7 == 0
    }

    pub fn is_present(&self, pos: Position) -> bool {
        self.get_nybble(pos) & 0x7 != 0
    }

    pub fn is_present_and_color(&self, pos: Position, color: PieceColor) -> bool {
        self.get(pos).is_some_and(|piece| piece.color == color)
    }

    pub fn is_present_and_kind(&self, pos: Position, kind: PieceKind) -> bool {
        self.get(pos).is_some_and(|piece| piece.kind == kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitBoard(pub u64);

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BitBoard> for u64 {
    fn from(value: BitBoard) -> Self {
        value.0
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);

    pub const RANK_1: Self = Self(0x00000000000000FF);
    pub const RANK_2: Self = Self(0x000000000000FF00);
    pub const RANK_3: Self = Self(0x0000000000FF0000);
    pub const RANK_4: Self = Self(0x00000000FF000000);
    pub const RANK_5: Self = Self(0x000000FF00000000);
    pub const RANK_6: Self = Self(0x0000FF0000000000);
    pub const RANK_7: Self = Self(0x00FF000000000000);
    pub const RANK_8: Self = Self(0xFF00000000000000);

    pub const FILE_A: Self = Self(0x0101010101010101);
    pub const FILE_B: Self = Self(0x0202020202020202);
    pub const FILE_C: Self = Self(0x0404040404040404);
    pub const FILE_D: Self = Self(0x0808080808080808);
    pub const FILE_E: Self = Self(0x1010101010101010);
    pub const FILE_F: Self = Self(0x2020202020202020);
    pub const FILE_G: Self = Self(0x4040404040404040);
    pub const FILE_H: Self = Self(0x8080808080808080);

    pub const fn new() -> Self {
        Self::EMPTY
    }

    pub const fn get(&self, pos: Position) -> bool {
        self.0 & (1u64 << (((pos.rank() << 3) + pos.file()))) != 0
    }

    pub const fn put(&mut self, pos: Position, val: bool) {
        if val {
            self.set(pos);
        } else {
            self.clear(pos);
        }
    }

    pub const fn clear(&mut self, pos: Position) {
        self.0 &= !(1u64 << (((pos.rank() << 3) + pos.file())));
    }

    pub const fn set(&mut self, pos: Position) {
        self.0 |= 1u64 << (((pos.rank() << 3) + pos.file()));
    }

    pub const fn count(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn iter_positions(self) -> BitBoardIter {
        BitBoardIter(self, 0, self.0.count_ones())
    }

    pub const fn iter_subsets(self) -> BitBoardSubsetIter {
        BitBoardSubsetIter {
            set: self,
            subset: Self::EMPTY,
            finished: false,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn shift(self, offset: Offset) -> Self {
        let shift = offset.into_shift();
        Self(if shift > 0 {
            self.0.rotate_left(shift.abs() as u32)
        } else if shift < 0 {
            self.0.rotate_right(shift.abs() as u32)
        } else {
            self.0
        })
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl std::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::fmt::Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in 0u8..8 {
            for file in 0u8..8 {
                let pos = Position::new(rank, file);
                if self.get(pos) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
                write!(f, " ")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

pub struct BitBoardIter(BitBoard, u32, u32);

impl BitBoardIter {
    pub const fn const_next(&mut self) -> Option<Position> {
        Some(loop {
            if self.1 >= 64 {
                return None;
            }
            let pos = Position::new((self.1 >> 3) as u8, (self.1 & 0x7) as u8);
            self.1 += 1;
            if self.0.get(pos) {
                break pos;
            }
        })
    }
}

impl Iterator for BitBoardIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        self.const_next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.2 as usize, Some(self.2 as usize))
    }
}

impl ExactSizeIterator for BitBoardIter {}

pub struct BitBoardSubsetIter {
    set: BitBoard,
    subset: BitBoard,
    finished: bool,
}

impl BitBoardSubsetIter {
    pub const fn const_next(&mut self) -> Option<BitBoard> {
        if self.finished {
            return None;
        }
        // Carry-Rippler trick: q' = (q - b) & b
        self.subset.0 = self.subset.0.wrapping_sub(self.set.0) & self.set.0;
        self.finished = self.subset.is_empty();
        Some(self.subset)
    }
}

impl Iterator for BitBoardSubsetIter {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        self.const_next()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PieceWiseBitBoard {
    pub pawns: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub rooks: BitBoard,
    pub queens: BitBoard,
    pub king: BitBoard,
}

impl PieceWiseBitBoard {
    pub fn bitboard_of_any(&self) -> BitBoard {
        self.pawns |
        self.knights |
        self.bishops |
        self.rooks |
        self.queens |
        self.king
    }

    pub fn bitboard_of(&self, kind: PieceKind) -> BitBoard {
        match kind {
            PieceKind::Pawn => self.pawns,
            PieceKind::Knight => self.knights,
            PieceKind::Bishop => self.bishops,
            PieceKind::Rook => self.rooks,
            PieceKind::Queen => self.queens,
            PieceKind::King => self.king,
        }
    }

    pub fn get(&self, pos: Position, kind: PieceKind) -> bool {
        match kind {
            PieceKind::Pawn => self.pawns.get(pos),
            PieceKind::Knight => self.knights.get(pos),
            PieceKind::Bishop => self.bishops.get(pos),
            PieceKind::Rook => self.rooks.get(pos),
            PieceKind::Queen => self.queens.get(pos),
            PieceKind::King => self.king.get(pos),
        }
    }

    pub fn put(&mut self, pos: Position, kind: PieceKind, val: bool) {
        match kind {
            PieceKind::Pawn => self.pawns.put(pos, val),
            PieceKind::Knight => self.knights.put(pos, val),
            PieceKind::Bishop => self.bishops.put(pos, val),
            PieceKind::Rook => self.rooks.put(pos, val),
            PieceKind::Queen => self.queens.put(pos, val),
            PieceKind::King => self.king.put(pos, val),
        }
    }

    pub fn clear(&mut self, pos: Position, kind: PieceKind) {
        match kind {
            PieceKind::Pawn => self.pawns.clear(pos),
            PieceKind::Knight => self.knights.clear(pos),
            PieceKind::Bishop => self.bishops.clear(pos),
            PieceKind::Rook => self.rooks.clear(pos),
            PieceKind::Queen => self.queens.clear(pos),
            PieceKind::King => self.king.clear(pos),
        }
    }

    pub fn set(&mut self, pos: Position, kind: PieceKind) {
        match kind {
            PieceKind::Pawn => self.pawns.set(pos),
            PieceKind::Knight => self.knights.set(pos),
            PieceKind::Bishop => self.bishops.set(pos),
            PieceKind::Rook => self.rooks.set(pos),
            PieceKind::Queen => self.queens.set(pos),
            PieceKind::King => self.king.set(pos),
        }
    }
}