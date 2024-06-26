use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MoveError {
    #[error("Illegal king side castle")]
    IllegalKingSideCastle,

    #[error("Illegal queen side castle")]
    IllegalQueenSideCastle,

    #[error("Illegal capture")]
    IllegalCapture,

    #[error("Unknown piece")]
    UnknownPiece,

    #[error("Ambiguous move notation")]
    AmbiguousMoveNotation,

    #[error("Can't find piece to move")]
    InvalidPieceToMove,

    #[error("Invalid promotion")]
    InvalidPromotion,
}
