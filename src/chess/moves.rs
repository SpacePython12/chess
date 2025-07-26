use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllegalReason {
    /// The piece tried to move in an invalid way.
    InvalidMove,
    /// The piece tried to capture a piece with the same color.
    SameColor,
    /// The piece tried to move while its king was in check.
    InCheck,
    /// The piece tried to move after the game was over.
    InCheckmate,
    /// The piece tried to move to the exact same spot it already was.
    NoMove,
    /// An empty slot tried to move.
    NoPiece,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveKind {
    Basic,
    EnPassant,
    Castle,
    PawnDoublePush,
    Promotion(PromotionKind),
}

impl MoveKind {
    pub fn is_promotion(&self) -> bool {
        match self {
            Self::Promotion(_) => true,
            _ => false
        }
    }

    pub fn promotion_kind(&self) -> Option<PromotionKind> {
        match self {
            Self::Promotion(kind) => Some(*kind),
            _ => None
        }
    }
}

impl super::IntoPacked for MoveKind {
    type Packed = u8;

    const MASK: Self::Packed = 0b111;

    fn into_packed(self) -> Self::Packed {
        match self {
            MoveKind::Basic => 0b000,
            MoveKind::EnPassant => 0b001,
            MoveKind::Castle => 0b010,
            MoveKind::PawnDoublePush => 0b011,
            // MoveKind::Promotion => 0b100,
            MoveKind::Promotion(promotion_kind) => match promotion_kind {
                PromotionKind::Queen => 0b100,
                PromotionKind::Knight => 0b101,
                PromotionKind::Rook => 0b110,
                PromotionKind::Bishop => 0b111,
            },
        }
    }
}

impl super::FromPacked for MoveKind {
    fn from_packed(packed: Self::Packed) -> Self {
        match packed {
            0b000 => MoveKind::Basic,
            0b001 => MoveKind::EnPassant,
            0b010 => MoveKind::Castle,
            0b011 => MoveKind::PawnDoublePush,
            // 0b100..=0b111 => MoveKind::Promotion,
            0b100 => MoveKind::Promotion(PromotionKind::Queen),
            0b101 => MoveKind::Promotion(PromotionKind::Knight),
            0b110 => MoveKind::Promotion(PromotionKind::Rook),
            0b111 => MoveKind::Promotion(PromotionKind::Bishop),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PromotionKind {
    Queen,
    Rook,
    Bishop,
    Knight,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u16);

impl Move {
    pub(super) fn new(src: position::Position, dst: position::Position, kind: MoveKind) -> Self {
        Self((src.into_packed() as u16) | ((dst.into_packed() as u16) << 6) | ((kind.into_packed() as u16) << 12))
    }

    #[inline]
    pub fn src(&self) -> position::Position {
        position::Position::from_packed((self.0 & 0x3F) as u8)
    }

    #[inline]
    pub fn dst(&self) -> position::Position {
        position::Position::from_packed(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline]
    pub fn kind(&self) -> MoveKind {
        MoveKind::from_packed(((self.0 >> 12) & 0x7) as u8)
    }

    pub fn piece(&self, board: &Board) -> Piece {
        board.get(self.src()).unwrap()
    }

    pub fn capture(&self, board: &Board) -> Option<Piece> {
        board.get(if self.kind() == MoveKind::EnPassant {
            let dst = self.dst();
            Position::new(match dst.rank() {
                5 => 4,
                2 => 3,
                _ => unreachable!()
            }, dst.file())
        } else {
            self.dst()
        })
    }

    pub fn is_promotion(&self) -> bool {
        self.kind().is_promotion()
    }

    pub fn promotion_kind(&self) -> Option<PromotionKind> {
        self.kind().promotion_kind()
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.src(), self.dst())?;
        if let Some(kind) = self.promotion_kind() {
            write!(f, "{}", match kind {
                PromotionKind::Queen => 'q',
                PromotionKind::Knight => 'n',
                PromotionKind::Rook => 'r',
                PromotionKind::Bishop => 'b',
            })?;
        }
        Ok(())
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.src().cmp(&other.src()).then(self.dst().cmp(&other.dst())).then(self.promotion_kind().cmp(&other.promotion_kind()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveKey(u16);

impl MoveKey {
    pub(super) fn new(src: position::Position, dst: position::Position, promotions: u8) -> Self {
        Self((src.into_packed() as u16) | ((dst.into_packed() as u16) << 6) | (((promotions & 0xF) as u16) << 12))
    }

    #[inline]
    pub fn src(&self) -> position::Position {
        position::Position::from_packed((self.0 & 0x3F) as u8)
    }

    #[inline]
    pub fn dst(&self) -> position::Position {
        position::Position::from_packed(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline]
    fn promotions(&self) -> u16 {
        (self.0 >> 12) & 0xF
    }

    #[inline]
    fn promotion_mask(kind: PromotionKind) -> u16 {
        match kind {
            PromotionKind::Queen => 0x1000,
            PromotionKind::Knight => 0x2000,
            PromotionKind::Rook => 0x4000,
            PromotionKind::Bishop => 0x8000,
        }
    }

    pub fn has_any_promotions(&self) -> bool {
        self.0 & 0xF000 != 0
    }

    pub fn has_promotion(&self, kind: PromotionKind) -> bool {
        self.0 & Self::promotion_mask(kind) != 0
    }

    pub fn set_all_promotions(&mut self) {
        self.0 |= 0xF000;
    }

    pub fn set_promotion(&mut self, kind: PromotionKind) {
        self.0 |= Self::promotion_mask(kind);
    }

    pub fn clear_all_promotions(&mut self) {
        self.0 &= !0xF000;
    }

    pub fn clear_promotion(&mut self, kind: PromotionKind) {
        self.0 &= !Self::promotion_mask(kind);
    }
}