pub mod bitboards;
pub mod pieces;
pub mod position;
pub mod board;
pub mod moves;
pub mod zobrist;
pub mod move_gen;
pub mod magic;
mod util;

pub use pieces::*;
pub use position::Position;
pub use board::Board;
pub use moves::{Move, PromotionKind, MoveKind};
pub use move_gen::{MoveGenerator, MoveTree};

pub trait IntoPacked: Sized + Copy + Clone {
    type Packed: Sized + Copy + Clone;
    const MASK: Self::Packed;

    fn into_packed(self) -> Self::Packed;
}

pub trait FromPacked: IntoPacked {
    fn from_packed(packed: Self::Packed) -> Self;
}

