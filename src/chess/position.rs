use crate::chess::bitboards::BitBoard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(u8);

impl From<(u8, u8)> for Position {
    fn from(value: (u8, u8)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl std::ops::Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(7-self.rank(), 7-self.file())
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.rank()+rhs.rank(), self.file()+rhs.file())
    }
}

impl std::ops::Sub for Position {
    type Output = Offset;

    fn sub(self, rhs: Self) -> Self::Output {
        Offset::new((self.rank() as i8) - (rhs.rank() as i8), (self.file() as i8) - (rhs.file() as i8))
    }
}

impl std::ops::Add<Offset> for Position {
    type Output = Self;

    fn add(self, rhs: Offset) -> Self::Output {
        Self::new(
            self.rank().wrapping_add_signed(rhs.rank_offset()), 
            self.file().wrapping_add_signed(rhs.file_offset())
        )
    }
}

impl std::ops::Sub<Offset> for Position {
    type Output = Self;

    fn sub(self, rhs: Offset) -> Self::Output {
        self + (-rhs)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (rank, file) = self.into_chars();
        write!(f, "{}{}", rank, file)
    }
}

impl Position {
    pub const A1: Self = Self::new(0, 0);
    pub const A2: Self = Self::new(1, 0);
    pub const A3: Self = Self::new(2, 0);
    pub const A4: Self = Self::new(3, 0);
    pub const A5: Self = Self::new(4, 0);
    pub const A6: Self = Self::new(5, 0);
    pub const A7: Self = Self::new(6, 0);
    pub const A8: Self = Self::new(7, 0);
    pub const B1: Self = Self::new(0, 1);
    pub const B2: Self = Self::new(1, 1);
    pub const B3: Self = Self::new(2, 1);
    pub const B4: Self = Self::new(3, 1);
    pub const B5: Self = Self::new(4, 1);
    pub const B6: Self = Self::new(5, 1);
    pub const B7: Self = Self::new(6, 1);
    pub const B8: Self = Self::new(7, 1);
    pub const C1: Self = Self::new(0, 2);
    pub const C2: Self = Self::new(1, 2);
    pub const C3: Self = Self::new(2, 2);
    pub const C4: Self = Self::new(3, 2);
    pub const C5: Self = Self::new(4, 2);
    pub const C6: Self = Self::new(5, 2);
    pub const C7: Self = Self::new(6, 2);
    pub const C8: Self = Self::new(7, 2);
    pub const D1: Self = Self::new(0, 3);
    pub const D2: Self = Self::new(1, 3);
    pub const D3: Self = Self::new(2, 3);
    pub const D4: Self = Self::new(3, 3);
    pub const D5: Self = Self::new(4, 3);
    pub const D6: Self = Self::new(5, 3);
    pub const D7: Self = Self::new(6, 3);
    pub const D8: Self = Self::new(7, 3);
    pub const E1: Self = Self::new(0, 4);
    pub const E2: Self = Self::new(1, 4);
    pub const E3: Self = Self::new(2, 4);
    pub const E4: Self = Self::new(3, 4);
    pub const E5: Self = Self::new(4, 4);
    pub const E6: Self = Self::new(5, 4);
    pub const E7: Self = Self::new(6, 4);
    pub const E8: Self = Self::new(7, 4);
    pub const F1: Self = Self::new(0, 5);
    pub const F2: Self = Self::new(1, 5);
    pub const F3: Self = Self::new(2, 5);
    pub const F4: Self = Self::new(3, 5);
    pub const F5: Self = Self::new(4, 5);
    pub const F6: Self = Self::new(5, 5);
    pub const F7: Self = Self::new(6, 5);
    pub const F8: Self = Self::new(7, 5);
    pub const G1: Self = Self::new(0, 6);
    pub const G2: Self = Self::new(1, 6);
    pub const G3: Self = Self::new(2, 6);
    pub const G4: Self = Self::new(3, 6);
    pub const G5: Self = Self::new(4, 6);
    pub const G6: Self = Self::new(5, 6);
    pub const G7: Self = Self::new(6, 6);
    pub const G8: Self = Self::new(7, 6);
    pub const H1: Self = Self::new(0, 7);
    pub const H2: Self = Self::new(1, 7);
    pub const H3: Self = Self::new(2, 7);
    pub const H4: Self = Self::new(3, 7);
    pub const H5: Self = Self::new(4, 7);
    pub const H6: Self = Self::new(5, 7);
    pub const H7: Self = Self::new(6, 7);
    pub const H8: Self = Self::new(7, 7);

    pub const NUM: usize = 64;

    pub const fn new(rank: u8, file: u8) -> Self {
        Self(((rank & 0b111) << 3) | (file & 0b111))
    }

    pub const fn rank(&self) -> u8 {
        (self.0 >> 3) & 0b111
    }

    pub const fn set_rank(&mut self, rank: u8) {
        self.0 &= 0b000_111;
        self.0 |= (rank & 0b111) << 3;
    }

    pub const fn file(&self) -> u8 {
        self.0 & 0b111
    }

    pub const fn set_file(&mut self, file: u8) {
        self.0 &= 0b111_000;
        self.0 |= file & 0b111;
    }

    pub const fn into_index(self) -> u8 {
        self.0
    }

    pub const fn from_index(index: u8) -> Self {
        Self(index & 0b111_111)
    }

    pub fn into_chars(self) -> (char, char) {
        const FILES: [char; 8] = [
            'a',
            'b',
            'c',
            'd',
            'e',
            'f',
            'g',
            'h'
        ];
        const RANKS: [char; 8] = [
            '1',
            '2',
            '3',
            '4',
            '5',
            '6',
            '7',
            '8'
        ];
        (FILES[self.file() as usize], RANKS[self.rank() as usize])
    }

    pub fn from_chars(rchar: char, fchar: char) -> Option<Self> {
        if let (Some(rank), Some(file)) = (
            if ('1'..='8').contains(&rchar) {
                Some((rchar as u8)-('1' as u8))
            } else { None },
            if ('a'..='h').contains(&fchar) {
                Some((fchar as u8)-('a' as u8))
            } else { None }
        ) { Some(Self::new(rank, file)) } else { None }
    }
    
    pub const fn wrapping_offset(self, offset: Offset) -> Self {
        let rank = self.rank().wrapping_add_signed(offset.rank_offset()) & 0b111;
        let file = self.file().wrapping_add_signed(offset.file_offset()) & 0b111;
        Self::new(rank, file)
    }

    pub const fn checked_offset(self, offset: Offset) -> Option<Self> {
        let rank = if let Some(rank) = self.rank().checked_add_signed(offset.rank_offset()) {
            if rank >= 8 {
                return None;
            }
            rank
        } else { return None; };

        let file = if let Some(file) = self.file().checked_add_signed(offset.file_offset()) {
            if file >= 8 {
                return None;
            }
            file
        } else { return None; };

        Some(Self::new(rank, file))
    }

    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self.0)
    }
}

impl super::IntoPacked for Position {
    type Packed = u8;

    const MASK: Self::Packed = 0b111111;

    fn into_packed(self) -> Self::Packed {
        self.into_index()
    }
}

impl super::FromPacked for Position {
    fn from_packed(packed: Self::Packed) -> Self {
        Self::from_index(packed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offset(u8);

impl std::ops::Neg for Offset {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0 ^ 0b1000_1000)
    }
}

impl std::ops::Mul<i8> for Offset {
    type Output = Self;

    fn mul(self, rhs: i8) -> Self::Output {
        Self::new(self.rank_offset()*rhs, self.file_offset()*rhs)
    }
}

impl From<(i8, i8)> for Offset {
    fn from(value: (i8, i8)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Offset {
    pub const N: Self =  Self::new( 1,  0);
    pub const S: Self =  Self::new(-1,  0);
    pub const W: Self =  Self::new( 0, -1);
    pub const E: Self =  Self::new( 0,  1);
    pub const NW: Self = Self::new( 1, -1);
    pub const SE: Self = Self::new(-1,  1);
    pub const NE: Self = Self::new( 1,  1);
    pub const SW: Self = Self::new(-1, -1);

    pub const ORTHO_DIRECTIONS: [Self; 4] = [
        Self::N,
        Self::S,
        Self::W,
        Self::E,
    ];

    pub const DIAG_DIRECTIONS: [Self; 4] = [
        Self::NW,
        Self::SE,
        Self::NE,
        Self::SW,
    ];

    pub const DIRECTIONS: [Self; 8] = [
        Self::N,
        Self::S,
        Self::W,
        Self::E,
        Self::NW,
        Self::SE,
        Self::NE,
        Self::SW,
    ];

    pub const fn new(rank: i8, file: i8) -> Self {
        let mut this = Self(0);
        this.set_rank_offset(rank);
        this.set_file_offset(file);
        this
    }

    pub const fn rank_offset(&self) -> i8 {
        if self.0 & 0b1000_0000 == 0 {
            ((self.0 >> 4) & 0b111) as i8
        } else {
            -(((self.0 >> 4) & 0b111) as i8)
        }
    }

    pub const fn set_rank_offset(&mut self, rank: i8) {
        self.0 &= 0b0000_1111;
        if rank < 0 {
            self.0 |= 0b1000_0000;
        }
        self.0 |= ((rank.abs() as u8) & 0b111) << 4
    }

    pub const fn file_offset(&self) -> i8 {
        if self.0 & 0b0000_1000 == 0 {
            (self.0 & 0b111) as i8
        } else {
            -((self.0 & 0b111) as i8)
        }
    }

    pub const fn set_file_offset(&mut self, file: i8) {
        self.0 &= 0b1111_0000;
        if file < 0 {
            self.0 |= 0b0000_1000;
        }
        self.0 |= (file.abs() as u8) & 0b111
    }

    pub const fn select_neg(&mut self, rank: bool, file: bool) {
        if rank {
            self.0 ^= 0b1000_0000;
        }
        if file {
            self.0 ^= 0b0000_1000;
        }
    }

    pub const fn is_ortho(&self) -> bool {
        (self.rank_offset() == 0) != (self.file_offset() == 0)
    }

    pub const fn is_diag(&self) -> bool {
        self.rank_offset().abs() == self.file_offset().abs()
    }

    pub const fn signum(self) -> Self {
        Self::new(self.rank_offset().signum(), self.file_offset().signum())
    }

    pub const fn into_shift(self) -> i8 {
        self.rank_offset() * 8 + self.file_offset()
    }

    pub const fn with_magnitude(self, rhs: i8) -> Self {
        Self::new(self.rank_offset()*rhs, self.file_offset()*rhs)
    }
}