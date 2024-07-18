use super::constants::*;
use super::errors::*;
use super::piece::*;
use super::square::*;
use super::utils::{self, *};

#[derive(Debug)]
pub struct Board {
    pub _board: Vec<Square>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            _board: vec![Square { piece: None }; BOARD_SIZE],
        }
    }

    pub fn ptr(&self) -> *const Square {
        self._board.as_ptr()
    }

    /// Return the PieceType and its associated value on a square. Return a ChessError if the index is out of range
    pub fn get(&self, sq: SquareCoordinate) -> ChessResult<Option<Piece>> {
        let idx = self.is_valid(sq.to_index())?;

        let sq = self._board.get(idx);

        if let Some(sq) = sq {
            return Ok(sq.piece);
        }

        return Err(ChessError::InvalidIndex(idx));
    }

    /// Put a piece at specific index on the board. Return the piece and index if succeed, or a ChessError if not.
    pub fn set(
        &mut self,
        sq: SquareCoordinate,
        piece_type: PieceType,
        color: Color,
    ) -> ChessResult<(Piece, usize)> {
        let idx = self.is_valid(sq.to_index())?;

        let piece = Piece { piece_type, color };

        self._board[idx].piece = Some(piece);

        Ok((piece, idx))
    }

    pub fn remove(&mut self, sq: SquareCoordinate) -> ChessResult<()> {
        let idx = self.is_valid(sq.to_index())?;

        self._board[idx].piece = None;

        Ok(())
    }

    /// Check if the index is exists on the board. Return the index if valid, ChessError if not.
    pub fn is_valid(&self, idx: usize) -> ChessResult<usize> {
        utils::is_valid(idx)
    }
}
