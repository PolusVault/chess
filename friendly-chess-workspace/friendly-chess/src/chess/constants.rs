pub const BOARD_SIZE: usize = 128;
pub const COLOR_MASK: u8 = 128; // 10000000

pub const WHITE_PAWN_DELTAS: [i8; 4] = [-16, -32, -17, -15];
pub const BLACK_PAWN_DELTAS: [i8; 4] = [16, 32, 17, 15];
pub const BISHOP_DELTAS: [i8; 4] = [17, 15, -17, -15];
pub const ROOK_DELTAS: [i8; 4] = [16, -16, 1, -1];
pub const QUEEN_DELTAS: [i8; 8] = [16, -16, 1, -1, 17, 15, -17, -15];
pub const KNIGHT_DELTAS: [i8; 8] = [14, 31, 18, 33, -14, -31, -18, -33];
pub const KING_DELTAS: [i8; 10] = [1, 16, 17, 15, -1, -16, -17, -15, 2, -2];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    WHITE = 0,
    BLACK = 128,
}

impl Color {
    pub fn to_value(&self) -> u8 {
        *self as u8
    }

    pub fn to_string(&self) -> String {
        if self.to_value() == 0 {
            return String::from("w");
        } else {
            return String::from("b");
        }
    }
}

// const MOVED_KING_DELTAS: [i8; 8] = [1, 16, 17, 15, -1, -16, -17, -15];

// why does it have 239 items?
// because of how the indexes of the squares on the real board are laid out due to the fact that
// we have to use some indexes in between to represent the dummy board
/*

  what is the signifance of the fact that the diff. between each square is unique?
  because they are unique, we can store every single diff. (offset by 119) in an array for lookup.
  This way, we can quickly check if a square can be attacked by just finding the difference between the indexes
*/

// 10110101
#[rustfmt::skip]
pub const ATTACKS: [u8; 239] = [
  20, 0, 0, 0, 0, 0, 0, 24,  0, 0, 0, 0, 0, 0,20, 0,
   0,20, 0, 0, 0, 0, 0, 24,  0, 0, 0, 0, 0,20, 0, 0,
   0, 0,20, 0, 0, 0, 0, 24,  0, 0, 0, 0,20, 0, 0, 0,
   0, 0, 0,20, 0, 0, 0, 24,  0, 0, 0,20, 0, 0, 0, 0,
   0, 0, 0, 0,20, 0, 0, 24,  0, 0,20, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0,20, 2, 24,  2,20, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0, 2,53, 56, 53, 2, 0, 0, 0, 0, 0, 0, // Note the zero in the very middle, it basically represents the current PieceType that is being evaluated for attacks
  24,24,24,24,24,24,56,  0, 56,24,24,24,24,24,24, 0, // But the PieceType isn't always in the middle? We can "move" it to the middle by adding 119
   0, 0, 0, 0, 0, 2,181,56,181, 2, 0, 0, 0, 0, 0, 0, // and then applying the difference between two squares to find the index relative to the PieceType in the middle
   0, 0, 0, 0, 0,20, 2, 24,  2,20, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0,20, 0, 0, 24,  0, 0,20, 0, 0, 0, 0, 0,
   0, 0, 0,20, 0, 0, 0, 24,  0, 0, 0,20, 0, 0, 0, 0,
   0, 0,20, 0, 0, 0, 0, 24,  0, 0, 0, 0,20, 0, 0, 0,
   0,20, 0, 0, 0, 0, 0, 24,  0, 0, 0, 0, 0,20, 0, 0,
  20, 0, 0, 0, 0, 0, 0, 24,  0, 0, 0, 0, 0, 0,20
];

pub const BOARD_MAP: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 16, 17, 18, 19, 20, 21, 22, 23, 32, 33, 34, 35, 36, 37, 38, 39, 48, 49,
    50, 51, 52, 53, 54, 55, 64, 65, 66, 67, 68, 69, 70, 71, 80, 81, 82, 83, 84, 85, 86, 87, 96, 97,
    98, 99, 100, 101, 102, 103, 112, 113, 114, 115, 116, 117, 118, 119,
];

pub const FILES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
