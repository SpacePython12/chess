#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceKind {
    Pawn = 1,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub const fn value(&self) -> i32 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 300,
            PieceKind::Bishop => 300,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 2000,
        }
    }
}

impl super::IntoPacked for PieceKind {
    type Packed = u8;

    const MASK: Self::Packed = 0x7;

    fn into_packed(self) -> Self::Packed {
        match self {
            PieceKind::Pawn => 0x1,
            PieceKind::Knight => 0x2,
            PieceKind::Bishop => 0x3,
            PieceKind::Rook => 0x4,
            PieceKind::Queen => 0x5,
            PieceKind::King => 0x6,
        }
    }
}

impl super::IntoPacked for Option<PieceKind> {
    type Packed = u8;

    const MASK: Self::Packed = 0x7;

    fn into_packed(self) -> Self::Packed {
        if let Some(piece_kind) = self {
            piece_kind.into_packed()
        } else {
            0
        }
    }
}

impl super::FromPacked for Option<PieceKind> {
    fn from_packed(packed: Self::Packed) -> Self {
        match packed & 0x7 {
            0x0 | 0x7 => None,
            0x1 => Some(PieceKind::Pawn),
            0x2 => Some(PieceKind::Knight),
            0x3 => Some(PieceKind::Bishop),
            0x4 => Some(PieceKind::Rook),
            0x5 => Some(PieceKind::Queen),
            0x6 => Some(PieceKind::King),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PieceColor {
    #[default]
    White,
    Black
}

impl std::ops::Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

impl PieceColor {
    pub const fn is_white(&self) -> bool {
        match self {
            PieceColor::White => true,
            PieceColor::Black => false,
        }
    }

    pub const fn is_black(&self) -> bool {
        !self.is_white()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor
}

impl Piece {
    pub const WHITE_PAWN: Self = Self::new(PieceKind::Pawn, PieceColor::White);
    pub const BLACK_PAWN: Self = Self::new(PieceKind::Pawn, PieceColor::Black);
    pub const WHITE_KNIGHT: Self = Self::new(PieceKind::Knight, PieceColor::White);
    pub const BLACK_KNIGHT: Self = Self::new(PieceKind::Knight, PieceColor::Black);
    pub const WHITE_BISHOP: Self = Self::new(PieceKind::Bishop, PieceColor::White);
    pub const BLACK_BISHOP: Self = Self::new(PieceKind::Bishop, PieceColor::Black);
    pub const WHITE_ROOK: Self = Self::new(PieceKind::Rook, PieceColor::White);
    pub const BLACK_ROOK: Self = Self::new(PieceKind::Rook, PieceColor::Black);
    pub const WHITE_QUEEN: Self = Self::new(PieceKind::Queen, PieceColor::White);
    pub const BLACK_QUEEN: Self = Self::new(PieceKind::Queen, PieceColor::Black);
    pub const WHITE_KING: Self = Self::new(PieceKind::King, PieceColor::White);
    pub const BLACK_KING: Self = Self::new(PieceKind::King, PieceColor::Black);

    pub const fn new(kind: PieceKind, color: PieceColor) -> Self {
        Self { kind, color }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        if let Some(kind) = match c.to_ascii_lowercase() {
            'p' => Some(PieceKind::Pawn),
            'n' => Some(PieceKind::Knight),
            'b' => Some(PieceKind::Bishop),
            'r' => Some(PieceKind::Rook),
            'q' => Some(PieceKind::Queen),
            'k' => Some(PieceKind::King),
            _ => None
        } {
            if c.is_ascii_uppercase() {
                Some(Self::new(kind, PieceColor::White))
            } else {
                Some(Self::new(kind, PieceColor::White))
            }
        } else {
            None
        }
    }

    pub const fn into_char(self) -> char {
        let ch = match self.kind {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };

        if self.color.is_white() {
            ch.to_ascii_uppercase()
        } else {
            ch
        }
    }

    pub const fn value(&self) -> i32 {
        self.kind.value()
    }
}

impl super::IntoPacked for Piece {
    type Packed = u8;

    const MASK: Self::Packed = 0xF;

    fn into_packed(self) -> Self::Packed {
        PieceKind::into_packed(self.kind) | ((self.color.is_black() as u8) << 3)
    }
}

impl super::IntoPacked for Option<Piece> {
    type Packed = u8;

    const MASK: Self::Packed = 0xF;

    fn into_packed(self) -> Self::Packed {
        if let Some(piece) = self {
            piece.into_packed()
        } else {
            0
        }
    }
}

impl super::FromPacked for Option<Piece> {
    fn from_packed(packed: Self::Packed) -> Self {
        match Option::<PieceKind>::from_packed(packed & 0x7) {
            Some(kind) => Some(Piece::new(kind, if packed & 0x8 == 0 {
                PieceColor::White
            } else {
                PieceColor::Black
            })),
            None => None
        }
    }
}