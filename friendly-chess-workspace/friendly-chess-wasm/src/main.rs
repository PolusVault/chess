use std::error::Error;

use friendly_chess::chess::{Chess, SquareCoordinate, *};
// use friendly_chess_wasm::j;


#[derive(Debug)]
pub struct PlayerMove {
    pub from: String,
    pub to: String,
    pub promotion_piece: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // let mut chess = ChessWasm::new();
    // chess.load_fen("8/5P2/k7/8/8/8/8/7K w - - 0 1".to_string());
    //
    // let moves = chess.moves_for_square("f7".to_string());
    //
    // match moves {
    //     Ok(m) =>println!("{:?}", m),
    //     Err(_) => println!("error")
    // }

    let mut chess = Chess::new();
    chess.load_fen("8/5P2/k7/8/8/8/8/7K w - - 0 1".to_string());

    let moves = chess.moves_for_square(SquareCoordinate::F7);

    let moves: Vec<PlayerMove> = chess
        .moves_for_square(SquareCoordinate::F7)?
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
    println!("{:?}", moves);
    Ok(())
}
