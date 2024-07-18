use super::constants;
use super::errors::*;

pub type ChessResult<T> = core::result::Result<T, ChessError>;

pub fn is_valid(idx: usize) -> ChessResult<usize> {
    if idx & 0x88 == 0 && idx < 256 {
        return Ok(idx);
    } else {
        return Err(ChessError::InvalidIndex(idx));
    }
}

pub fn convert_index_to_algebraic_notation(index: u8) -> String {
    let file = index & 7;
    let rank = 8 - ((index >> 4) + 1) + 1;

    let file_letter = constants::FILES[file as usize];

    let mut notation = String::new();

    notation.push_str(file_letter);
    notation.push_str(rank.to_string().as_str());

    notation
}

pub fn convert_algebraic_notation_to_index(notation: &str) -> u8 {
    let mut parts = notation.chars();

    let file = parts.next().unwrap();
    let rank = (parts.next().unwrap().to_digit(10).unwrap() - 1) as u8;

    let file = constants::FILES
        .iter()
        .position(|f| f.eq(&file.to_string().as_str()))
        .unwrap() as u8;

    // we minus rank from 7 because the board is reversed (upside down)
    // so for example, for "e7", the rank 7 is rank 1 on our board
    (8 * (7 - rank) + file) as u8
}
