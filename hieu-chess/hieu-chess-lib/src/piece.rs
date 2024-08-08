use crate::chess::Color;
use crate::error::Error;
use std::convert::Into;

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum PType {
    PAWN = 1,   // 0000 0001
    KNIGHT = 2, // 0000 0010
    BISHOP = 4, // 0000 0100
    ROOK = 8,   // 0000 1000
    QUEEN = 16, // 0001 0000
    KING = 32,  // 0010 0000
}

#[derive(PartialEq, Debug, Clone)]
pub struct Piece {
    pub p_type: PType,
    pub color: Color,
}

impl Piece {
    pub fn new(p_type: PType, color: Color) -> Self {
        Self { p_type, color }
    }
}

impl TryFrom<&str> for Piece {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use PType::*;

        let color = if value == value.to_uppercase() {
            Color::WHITE
        } else {
            Color::BLACK
        };

        let t = match value.to_lowercase().as_str() {
            "p" | "pawn" => PAWN,
            "n" | "knight" => KNIGHT,
            "b" | "bishop" => BISHOP,
            "r" | "rook" => ROOK,
            "q" | "queen" => QUEEN,
            "k" | "king" => KING,
            _ => return Err(Error::InvalidPiece),
        };

        Ok(Piece::new(t, color))
    }
}

impl Into<String> for Piece {
    fn into(self) -> String {
        use PType::*;

        let t = match self.p_type {
            PAWN => "p",
            KNIGHT => "n",
            BISHOP => "b",
            ROOK => "r",
            QUEEN => "q",
            KING => "k",
        };

        if self.color == Color::WHITE {
            t.to_uppercase()
        } else {
            t.to_string()
        }
    }
}

// just hardcoding white as the color for now, it'll be changed accordingly
pub const PROMOTION_PIECES: [Piece; 4] = [
    Piece {
        p_type: PType::BISHOP,
        color: Color::WHITE,
    },
    Piece {
        p_type: PType::KNIGHT,
        color: Color::WHITE,
    },
    Piece {
        p_type: PType::ROOK,
        color: Color::WHITE,
    },
    Piece {
        p_type: PType::QUEEN,
        color: Color::WHITE,
    },
];
