use super::{piece::Piece, SquareCoordinate};

#[derive(Clone, PartialEq, Copy, Debug)]
#[repr(u8)]
pub enum MoveType {
    Normal = 0,
    EnPassantMove = 1,
    Capture = 2,
    EnPassantCapture = 4,
    CastleKingside = 8,
    CastleQueenside = 16,
    Promotion = 32,
}

// impl MoveType {
//     // Convert a Move type to its associated value
//     // pub fn to_value(&self) -> u8 {
//     //     *self as u8
//     // }
// }

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: SquareCoordinate,
    pub to: SquareCoordinate,
    pub promotion_piece: Option<Piece>,
}

#[derive(Clone, Debug)]
/// Represent a player move.
pub struct InternalMove {
    pub move_type: MoveType,

    pub from_sq: SquareCoordinate,
    pub from_piece: Piece,
    pub to_sq: SquareCoordinate,
    pub to_piece: Option<Piece>,
    pub promotion_piece: Option<Piece>,
}

// impl InternalMove {
//     pub fn new() -> Self {
//         Self {
//             move_type: MoveType::Normal,

//         }
//     }
// }
