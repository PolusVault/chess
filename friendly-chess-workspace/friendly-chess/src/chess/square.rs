use super::piece::*;
use super::utils::{self, *};

#[derive(Debug, Clone, Copy)]
pub struct Square {
    pub piece: Option<Piece>,
}

#[rustfmt::skip]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SquareCoordinate {
    A8 = 0,   B8 = 1,   C8 = 2,     D8 = 3,     E8 = 4,     F8 = 5,     G8 = 6,     H8 = 7,
    A7 = 16,  B7 = 17,  C7 = 18,    D7 = 19,    E7 = 20,    F7 = 21,    G7 = 22,    H7 = 23,
    A6 = 32,  B6 = 33,  C6 = 34,    D6 = 35,    E6 = 36,    F6 = 37,    G6 = 38,    H6 = 39,
    A5 = 48,  B5 = 49,  C5 = 50,    D5 = 51,    E5 = 52,    F5 = 53,    G5 = 54,    H5 = 55,
    A4 = 64,  B4 = 65,  C4 = 66,    D4 = 67,    E4 = 68,    F4 = 69,    G4 = 70,    H4 = 71,
    A3 = 80,  B3 = 81,  C3 = 82,    D3 = 83,    E3 = 84,    F3 = 85,    G3 = 86,    H3 = 87,
    A2 = 96,  B2 = 97,  C2 = 98,    D2 = 99,    E2 = 100,   F2 = 101,   G2 = 102,   H2 = 103,
    A1 = 112, B1 = 113, C1 = 114,   D1 = 115,   E1 = 116,   F1 = 117,   G1 = 118,   H1 = 119,


    __BAD_COORD = 200
}

impl SquareCoordinate {
    /// Convert a `Square` enum to its associated value (A8 = 0, B8 = 1, etc.)
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert square to standard algebraic notation
    pub fn to_san(&self) -> String {
        convert_index_to_algebraic_notation(self.to_index() as u8)
    }

    pub fn is_light_sq(&self) -> bool {
        let rank = self.rank();
        let file = self.file();
        (rank % 2 == 0 && file % 2 != 0) || (rank % 2 != 0 && file % 2 == 0)
    }

    pub fn is_dark_sq(&self) -> bool {
        let rank = self.rank();
        let file = self.file();
        (rank % 2 != 0 && file % 2 != 0) || (rank % 2 == 0 && file % 2 == 0)
    }

    pub fn rank(&self) -> u8 {
        let idx = self.to_index() as u8;

        8 - ((idx >> 4) + 1) + 1
    }

    pub fn file(&self) -> u8 {
        let idx = self.to_index() as u8;

        idx & 7
    }

    pub fn above(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() - 16)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn below(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() + 16)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn left(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() - 1)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn right(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() + 1)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn upper_left(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() - 17)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn upper_right(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() - 15)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn lower_left(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() + 15)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn lower_right(&self) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() + 17)? as u8;

        Ok(idx.to_coordinate())
    }

    pub fn subtract(&self, rhs: usize) -> ChessResult<Self> {
        let idx = utils::is_valid(self.to_index() - rhs)? as u8;

        Ok(idx.to_coordinate())
    }
}

pub trait SquareCoordinateExt {
    fn to_coordinate(&self) -> SquareCoordinate;
}

// I made a little script to generate this, I am not crazy
impl SquareCoordinateExt for u8 {
    fn to_coordinate(&self) -> SquareCoordinate {
        use SquareCoordinate::*;

        match *self {
            0 => A8,
            1 => B8,
            2 => C8,
            3 => D8,
            4 => E8,
            5 => F8,
            6 => G8,
            7 => H8,
            16 => A7,
            17 => B7,
            18 => C7,
            19 => D7,
            20 => E7,
            21 => F7,
            22 => G7,
            23 => H7,
            32 => A6,
            33 => B6,
            34 => C6,
            35 => D6,
            36 => E6,
            37 => F6,
            38 => G6,
            39 => H6,
            48 => A5,
            49 => B5,
            50 => C5,
            51 => D5,
            52 => E5,
            53 => F5,
            54 => G5,
            55 => H5,
            64 => A4,
            65 => B4,
            66 => C4,
            67 => D4,
            68 => E4,
            69 => F4,
            70 => G4,
            71 => H4,
            80 => A3,
            81 => B3,
            82 => C3,
            83 => D3,
            84 => E3,
            85 => F3,
            86 => G3,
            87 => H3,
            96 => A2,
            97 => B2,
            98 => C2,
            99 => D2,
            100 => E2,
            101 => F2,
            102 => G2,
            103 => H2,
            112 => A1,
            113 => B1,
            114 => C1,
            115 => D1,
            116 => E1,
            117 => F1,
            118 => G1,
            119 => H1,
            _ => __BAD_COORD,
        }
    }
}
