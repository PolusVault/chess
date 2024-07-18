use super::{
    castling::{Castling, CastlingRights},
    play_move::InternalMove,
    Color, Kings, Piece, PieceType, SquareCoordinate,
};
#[derive(Debug)]
pub struct HistoryEntry {
    pub player_move: InternalMove,
    pub turn: Color,

    pub kings: Kings,
    pub castling_rights: CastlingRights,

    pub en_passant_sq: Option<SquareCoordinate>,
}

#[derive(Debug)]
pub struct MoveHistory {
    pub entries: Vec<HistoryEntry>,
}

impl MoveHistory {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn push(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
    }

    pub fn pop(&mut self) -> Option<HistoryEntry> {
        self.entries.pop()
    }
}
