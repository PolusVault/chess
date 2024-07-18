// use std::error::Errsor;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ChessError {
    #[error("Illegal king side castle")]
    IllegalKingSideCastle,

    #[error("Illegal queen side castle")]
    IllegalQueenSideCastle,

    #[error("Illegal capture")]
    IllegalCapture,

    #[error("Unknown PieceType")]
    UnknownPiece,

    #[error("Ambiguous move notation")]
    AmbiguousMoveNotation,

    #[error("Invalid move (from: {0} to {1})")]
    InvalidMove(usize, usize),

    #[error("Invalid promotion")]
    InvalidPromotion,

    #[error("Invalid index {0}")]
    InvalidIndex(usize),

    #[error("Invalid PieceType type")]
    InvalidPieceType,

    #[error("Unexpected error: {0}")]
    UnknownError(String),
}
