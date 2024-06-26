use super::{utils::ChessResult, Board, Color, Kings, PieceType, SquareCoordinate};

#[derive(Clone, Debug)]
pub struct Castling {
    pub kingside: bool,
    pub queenside: bool,
}

#[derive(Clone, Debug)]
pub struct CastlingRights {
    pub white: Castling,
    pub black: Castling,
}

impl CastlingRights {
    pub fn new() -> Self {
        Self {
            white: Castling {
                kingside: true,
                queenside: true,
            },
            black: Castling {
                kingside: true,
                queenside: true,
            },
        }
    }

    pub fn update(&mut self, kings: &Kings, board: &Board) -> ChessResult<()> {
        if kings.white != Some(SquareCoordinate::E1) {
            self.white.kingside = false;
            self.white.queenside = false;
        }

        if kings.black != Some(SquareCoordinate::E8) {
            self.black.kingside = false;
            self.black.queenside = false;
        }

        let h1 = board.get(SquareCoordinate::H1)?;
        let a1 = board.get(SquareCoordinate::A1)?;
        let a8 = board.get(SquareCoordinate::A8)?;
        let h8 = board.get(SquareCoordinate::H8)?;

        // check for a kingside white rook, if it is not the a white rook at H1, then white can't castle kingside
        if let Some(piece) = h1 {
            if piece.piece_type != PieceType::ROOK && piece.color != Color::WHITE {
                self.white.kingside = false;
            }
        } else {
            // if no piece is present, we can't castle kingside, and so on...
            self.white.kingside = false;
        }

        // queenside white rook
        if let Some(piece) = a1 {
            if piece.piece_type != PieceType::ROOK && piece.color != Color::WHITE {
                self.white.queenside = false;
            }
        } else {
            self.white.queenside = false;
        }

        // kingside black rook
        if let Some(piece) = h8 {
            if piece.piece_type != PieceType::ROOK && piece.color != Color::BLACK {
                self.black.kingside = false;
            }
        } else {
            self.black.kingside = false;
        }

        // queenside black rook
        if let Some(piece) = a8 {
            if piece.piece_type != PieceType::ROOK && piece.color != Color::BLACK {
                self.black.queenside = false;
            }
        } else {
            self.black.queenside = false;
        }

        Ok(())
    }
}
