use serde::*;

use crate::websocket::messages::MakeMove;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Color {
    #[serde(rename(serialize = "b", deserialize = "b"))]
    Black,
    #[serde(rename(serialize = "w", deserialize = "w"))]
    White,
    #[serde(skip_serializing, skip_deserializing)]
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChessMove {
    pub from: String,
    pub to: String,
    pub promotion_piece: Option<String>,
}

impl From<MakeMove> for ChessMove {
    fn from(value: MakeMove) -> Self {
        Self {
            from: value.from,
            to: value.to,
            promotion_piece: value.promotion_piece,
        }
    }
}
