use crate::chess::Color;
use crate::error::Error;
use crate::piece::Piece;

pub fn is_valid_idx(idx: u8) -> Result<(), Error> {
    if idx & 0x88 == 0 {
        Ok(())
    } else {
        Err(Error::IllegalIndex)
    }
}

pub fn is_enemy(piece: &Piece, side_to_move: Color) -> bool {
    !is_friendly(piece, side_to_move)
}

pub fn is_friendly(piece: &Piece, side_to_move: Color) -> bool {
    piece.color == side_to_move
}
