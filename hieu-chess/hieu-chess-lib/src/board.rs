use crate::piece::Piece;

pub const SIZE_0X88: usize = 128;

#[derive(Clone)]
pub struct Board {
    _board: [Option<Piece>; SIZE_0X88],
}

const ARRAY_REPEAT_VALUE: Option<Piece> = None;

impl Board {
    pub fn new() -> Self {
        Self {
            _board: [ARRAY_REPEAT_VALUE; SIZE_0X88],
        }
    }

    pub fn set(&mut self, piece: Piece, idx: &u8) {
        self._board[*idx as usize] = Some(piece);
    }

    pub fn remove(&mut self, idx: &u8) {
        self._board[*idx as usize] = None;
    }

    pub fn get(&self, idx: &u8) -> Option<&Piece> {
        return self._board[*idx as usize].as_ref();
    }

    pub fn get_mut(&mut self, idx: &u8) -> Option<&mut Piece> {
        return self._board[*idx as usize].as_mut();
    }

    pub fn clear(&mut self) {
        self._board = [ARRAY_REPEAT_VALUE; SIZE_0X88];
    }

    pub fn get_internal_board(&self) -> &[Option<Piece>; SIZE_0X88] {
        &self._board
    }

    pub fn get_board_ptr(&self) -> *const Option<Piece> {
        self._board.as_ptr()
    }
}
