mod utils;

use friendly_chess::chess::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

#[derive(Serialize, Deserialize)]
pub struct PlayerMove {
    pub from: String,
    pub to: String,
    pub promotion_piece: Option<String>,
}

#[wasm_bindgen]
pub struct ChessWasm {
    chess: Chess,
}

#[wasm_bindgen]
impl ChessWasm {
    pub fn new() -> Self {
        Self {
            chess: Chess::new(),
        }
    }

    pub fn board(&self) -> *const Square {
        self.chess.board.ptr()
    }

    // pub fn board(&self) -> Result<JsValue, JsValue> {
    //     Ok(serde_wasm_bindgen::to_value(&self.chess.board._board)?);
    // }

    pub fn play_move(&mut self, player_move: JsValue) -> Result<(), JsError> {
        let player_move: PlayerMove = serde_wasm_bindgen::from_value(player_move).unwrap();

        // convert san to index and then coord
        let from = BOARD_MAP[convert_algebraic_notation_to_index(&player_move.from) as usize]
            .to_coordinate();
        let to = BOARD_MAP[convert_algebraic_notation_to_index(&player_move.to) as usize]
            .to_coordinate();

        let promotion_piece = match player_move.promotion_piece {
            Some(p) => Some(Piece {
                piece_type: PieceType::from_string_verbose(p.as_str())
                    .ok_or(JsError::new("Failed to parse promotion piece type"))?,
                color: self.chess.turn,
            }),
            None => None,
        };

        match self.chess.play_move(Move {
            from,
            to,
            promotion_piece,
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsError::new(&e.to_string())),
        }
    }

    pub fn is_checkmate(&mut self) -> Result<bool, JsError> {
        match self.chess.is_checkmate() {
            Ok(b) => Ok(b),
            Err(_) => Err(JsError::new("failed to check checkmate"))
        } 
    }

    pub fn is_draw(&mut self) -> Result<bool, JsError> {
        match self.chess.is_draw() {
            Ok(b) => Ok(b),
            Err(_) => Err(JsError::new("failed to check draw"))
        } 
    }

    pub fn is_stalemate(&mut self) -> Result<bool, JsError> {
        match self.chess.is_stalemate() {
            Ok(b) => Ok(b),
            Err(_) => Err(JsError::new("failed to check stalemate"))
        } 
    }

    pub fn is_insufficient_materials(&mut self) -> Result<bool, JsError> {
        match self.chess.is_insufficient_material() {
            Ok(b) => Ok(b),
            Err(_) => Err(JsError::new("failed to check materials"))
        } 
    }

    pub fn is_repetition(&mut self) -> Result<bool, JsError> {
        match self.chess.is_threefold_repetition() {
            Ok(b) => Ok(b),
            Err(_) => Err(JsError::new("failed to check repetition"))
        } 
    }


    pub fn moves_for_square(&mut self, square: String) -> Result<JsValue, JsError> {
        let square =
            BOARD_MAP[convert_algebraic_notation_to_index(&square) as usize].to_coordinate();

        let moves: Vec<PlayerMove> = self
            .chess
            .moves_for_square(square)?
            .iter()
            .map(|m| PlayerMove {
                from: convert_index_to_algebraic_notation(m.from.to_index() as u8),
                to: convert_index_to_algebraic_notation(m.to.to_index() as u8),
                promotion_piece: match m.promotion_piece {
                    Some(p) => Some(p.piece_type.to_string_verbose().unwrap().to_string()),
                    None => None,
                },
            })
            .collect();

        Ok(serde_wasm_bindgen::to_value(&moves)?)
    }

    pub fn load_fen(&mut self, fen: String) -> Result<(), JsError> {
        match self.chess.load_fen(fen) {
            Ok(_) => Ok(()),
            Err(_) => Err(JsError::new("Failed to load FEN")),
        }
    }

    pub fn get_fen(&self) -> Result<String, JsError> {
        match self.chess.get_fen() {
            Ok(fen) => Ok(fen),
            Err(_) => Err(JsError::new("Failed to get FEN")),
        }
    }

    pub fn turn(&self) -> String {
        self.chess.turn.to_string()
    }

    pub fn set_turn(&mut self, turn: &str) { 
        if turn == "w" {
            self.chess.set_turn(Color::WHITE);
        } else {
            self.chess.set_turn(Color::BLACK);
        }
    }
}

// #[wasm_bindgen]
// pub fn greet() {
//     alert("Hello, friendly-chess-wasm!");
// }
