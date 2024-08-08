use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("the move is illegal")]
    IllegalMove,

    #[error("no piece to move")]
    UnknownMove,

    #[error("not your turn to move")]
    MustWaitForTurn,

    #[error("invalid square index")]
    IllegalIndex,

    #[error("invalid piece string")]
    InvalidPiece,

    #[error("invalid square string")]
    InvalidSquareString,

    #[error("invalid color")]
    InvalidColor,
}
