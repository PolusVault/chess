mod board;
mod castling;
mod constants;
mod errors;
mod history;
mod piece;
mod play_move;
mod square;
mod utils;

use board::*;
use castling::*;
use errors::*;
use history::*;
use piece::*;
use play_move::*;
use std::collections::HashMap;
use std::ops::Range;
use utils::*;

pub use constants::{Color, BOARD_MAP, FILES};
pub use piece::{Piece, PieceType};
pub use play_move::Move;
pub use square::{Square, SquareCoordinate, SquareCoordinateExt};
pub use utils::{
    convert_algebraic_notation_to_index, convert_index_to_algebraic_notation, is_valid,
};

use self::constants::{
    BISHOP_DELTAS, BLACK_PAWN_DELTAS, BOARD_SIZE, COLOR_MASK, KING_DELTAS, KNIGHT_DELTAS,
    QUEEN_DELTAS, ROOK_DELTAS, WHITE_PAWN_DELTAS,
};

#[derive(Clone, Debug)]
pub struct Kings {
    pub white: Option<SquareCoordinate>,
    pub black: Option<SquareCoordinate>,
}

pub struct Chess {
    pub board: Board,
    pub turn: Color,

    /// the kings' positions on the board
    pub kings: Kings,
    pub castling_rights: CastlingRights,
    pub history: MoveHistory,
    pub white_captures: Vec<Piece>,
    pub black_captures: Vec<Piece>,

    // /// Record each unique positions on the board.
    // /// If any position occurs 3 times at any point in time, the game is declared draw
    unique_positions: HashMap<String, u8>,
    pub half_moves: u8,
    pub full_moves: u8,
    pub en_passant_sq: Option<SquareCoordinate>,
}

impl Chess {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Color::WHITE,
            kings: Kings {
                white: None,
                black: None,
            },
            history: MoveHistory::new(),
            castling_rights: CastlingRights::new(),
            en_passant_sq: None,
            white_captures: vec![],
            black_captures: vec![],
            full_moves: 0,
            half_moves: 0,
            unique_positions: HashMap::new(),
        }
    }

    /// Make a move on the board
    pub fn play_move(&mut self, m: Move) -> ChessResult<()> {
        // TODO: check if it is a legal move 
        self.make_move(m)?;
        self.change_turn();

        Ok(())
    }

    pub fn make_move(&mut self, m: Move) -> ChessResult<()> {
        let m = self.convert_to_internal_move(m)?;

        let history_entry = HistoryEntry {
            player_move: m.clone(),
            turn: self.turn,
            kings: self.kings.clone(),
            castling_rights: self.castling_rights.clone(),
            en_passant_sq: self.en_passant_sq,
        };

        self.half_moves += 1;

        self.set(m.to_sq, m.from_piece.piece_type, m.from_piece.color)?;
        self.remove(m.from_sq)?;

        if m.move_type == MoveType::Capture {
            if let Some(to_piece) = m.to_piece {
                if to_piece.color == Color::BLACK {
                    self.white_captures.push(to_piece);
                } else {
                    self.black_captures.push(to_piece);
                }
            }
        }

        if m.move_type == MoveType::EnPassantCapture {
            if let Some(en_passant_sq) = self.en_passant_sq {
                if m.from_piece.color == Color::WHITE {
                    self.white_captures.push(Piece {
                        piece_type: PieceType::PAWN,
                        color: Color::BLACK,
                    });
                    // remove the piece below
                    self.remove(en_passant_sq.below()?)?;
                } else {
                    self.black_captures.push(Piece {
                        piece_type: PieceType::PAWN,
                        color: Color::WHITE,
                    });
                    // remove the piece above
                    self.remove(en_passant_sq.above()?)?;
                }
            }
        }

        if m.move_type == MoveType::EnPassantMove {
            if m.from_piece.color == Color::WHITE {
                self.en_passant_sq = Some(m.to_sq.below()?);
            } else {
                self.en_passant_sq = Some(m.to_sq.above()?);
            }
        } else {
            self.en_passant_sq = None;
        }

        if m.move_type == MoveType::CastleKingside {
            self.remove(m.to_sq.right()?)?;
            self.set(m.to_sq.left()?, PieceType::ROOK, m.from_piece.color)?;
        }

        if m.move_type == MoveType::CastleQueenside {
            self.remove(m.to_sq.subtract(2)?)?;
            self.set(m.to_sq.right()?, PieceType::ROOK, m.from_piece.color)?;
        }

        if m.move_type == MoveType::Promotion {
            let promotion_piece = m
                .promotion_piece
                .ok_or(ChessError::UnknownError(format!("yah yeet")))?;
            self.set(m.to_sq, promotion_piece.piece_type, promotion_piece.color)?;
        }

        self.history.push(history_entry);
        self.castling_rights.update(&self.kings, &self.board)?;

        // reset half moves if it is a pawn move or a piece is captured
        if m.move_type == MoveType::Capture || m.from_piece.piece_type == PieceType::PAWN {
            self.half_moves = 0;
        }

        Ok(())
    }

    pub fn undo_move(&mut self) -> ChessResult<()> {
        if let Some(old) = self.history.pop() {
            self.turn = old.turn;
            self.en_passant_sq = old.en_passant_sq;
            self.kings = old.kings;
            self.castling_rights = old.castling_rights;

            let m = old.player_move;

            // put the piece back to its original square
            self.set(m.from_sq, m.from_piece.piece_type, m.from_piece.color)?;
            self.remove(m.to_sq)?;

            self.half_moves = self.half_moves.saturating_sub(1);

            if old.turn == Color::BLACK {
                self.full_moves = self.full_moves.saturating_sub(1);
            }

            if m.move_type == MoveType::Capture || m.move_type == MoveType::EnPassantCapture {
                if m.from_piece.color == Color::WHITE {
                    self.white_captures.pop();
                } else {
                    self.black_captures.pop();
                }
            }

            if m.move_type == MoveType::EnPassantCapture {
                // put the captured piece back
                if m.from_piece.color == Color::WHITE {
                    self.set(m.to_sq.below()?, PieceType::PAWN, Color::BLACK)?;
                } else {
                    self.set(m.to_sq.above()?, PieceType::PAWN, Color::WHITE)?;
                }
            }

            if m.move_type == MoveType::CastleKingside {
                // put the rooks back
                self.remove(m.to_sq.left()?)?;
                self.set(m.to_sq.right()?, PieceType::ROOK, m.from_piece.color)?;
            }

            if m.move_type == MoveType::CastleQueenside {
                // put the rooks back
                self.remove(m.to_sq.right()?)?;
                // put the rooks back two square to the left
                self.set(m.to_sq.subtract(2)?, PieceType::ROOK, m.from_piece.color)?;
            }

            if m.move_type == MoveType::Capture || m.to_piece.is_some() {
                if let Some(to_piece) = m.to_piece {
                    // put the captured piece back
                    self.set(m.to_sq, to_piece.piece_type, to_piece.color)?;
                }
            }

            // reset half moves if it is a pawn move or a piece is captured
            if m.move_type == MoveType::Capture || m.from_piece.piece_type == PieceType::PAWN {
                self.half_moves = 0;
            }
        }

        Ok(())
    }

    /// Get all legal moves for a specific piece type
    pub fn moves_for_piece_type() {}

    /// Get all legal moves for the piece on the specified square
    pub fn moves_for_square(&mut self, sq: SquareCoordinate) -> ChessResult<Vec<Move>> {
        let piece = self.get(sq)?.ok_or(ChessError::UnknownError(
            "Can't generate moves for empty square".to_string(),
        ))?;
        
        if piece.color != self.turn {
            return Ok(vec![]);
        }

        let mut legal_moves = vec![];
        let pseudo_legal_moves = match piece.piece_type {
            PieceType::KING => self.king_moves(sq),
            PieceType::QUEEN => self.sliding_moves(sq, QUEEN_DELTAS.to_vec()),
            PieceType::ROOK => self.sliding_moves(sq, ROOK_DELTAS.to_vec()),
            PieceType::BISHOP => self.sliding_moves(sq, BISHOP_DELTAS.to_vec()),
            PieceType::KNIGHT => self.knight_moves(sq),
            PieceType::PAWN => self.pawn_moves(sq),
        }?;

        for _move in pseudo_legal_moves {
            self.make_move(_move)?;
            if !self.in_check()? {
                legal_moves.push(_move);
            }

            self.undo_move()?;
        }

        Ok(legal_moves)
    }

    /// Get all legal moves for the player to move
    pub fn moves(&mut self) -> ChessResult<Vec<Move>> {
        let mut moves = vec![];

        for idx in 0..BOARD_SIZE {
            if let Ok(idx) = self.board.is_valid(idx) {
                if let Some(piece) = self.get((idx as u8).to_coordinate())? {
                    if self.is_friendly(piece) {
                        let mut a = self.moves_for_square((idx as u8).to_coordinate())?;
                        moves.append(&mut a);
                    }
                }
            }
        }

        Ok(moves)
    }

    /// Return true if the current player to move is in check
    pub fn in_check(&self) -> ChessResult<bool> {
        if self.turn == Color::WHITE {
            if let Some(king) = self.kings.white {
                return self.is_attacked(king);
            }
        } else {
            if let Some(king) = self.kings.black {
                return self.is_attacked(king);
            }
        }
        Ok(false)
    }

    pub fn is_checkmate(&mut self) -> ChessResult<bool> {
        let mut no_legal_moves = true;

        for idx in 0..BOARD_SIZE {
            let idx = match self.board.is_valid(idx) {
                Ok(idx) => idx,
                Err(_) => continue,
            } as u8;

            if let Some(piece) = self.get(idx.to_coordinate())? {
                if self.is_friendly(piece) {
                    let moves = self.moves_for_square(idx.to_coordinate())?;

                    if moves.len() > 0 {
                        no_legal_moves = false;
                    }
                }
            }
        }

        Ok(self.in_check()? && no_legal_moves)
    }

    pub fn is_draw(&mut self) -> ChessResult<bool> {
        Ok(self.is_stalemate()?
            || self.is_threefold_repetition()?
            || self.is_50_moves()?
            || self.is_insufficient_material()?)
    }

    pub fn is_stalemate(&mut self) -> ChessResult<bool> {
        let mut no_legal_moves = true;

        for idx in 0..BOARD_SIZE {
            let idx = match self.board.is_valid(idx) {
                Ok(idx) => idx,
                Err(_) => continue,
            } as u8;

            if let Some(piece) = self.get(idx.to_coordinate())? {
                if self.is_friendly(piece) {
                    let moves = self.moves()?;

                    if moves.len() > 0 {
                        no_legal_moves = false;
                    }
                }
            }
        }

        Ok(!self.in_check()? && no_legal_moves)
    }

    pub fn is_insufficient_material(&self) -> ChessResult<bool> {
        let mut friendly_knights = 0;
        let mut friendly_bishops = 0;
        let mut friendly_dark_bishops = 0;
        let mut friendly_light_bishops = 0;
        let mut enemy_dark_bishops = 0;
        let mut enemy_light_bishops = 0;
        let mut enemy_knights = 0;
        let mut enemy_bishops = 0;

        for idx in 0..BOARD_SIZE {
            let idx = match self.board.is_valid(idx) {
                Ok(idx) => idx,
                Err(_) => continue,
            } as u8;
            let coord = idx.to_coordinate();

            // if rank is odd and file is odd = dark
            // if rank is even and file is even = dark

            // if rank is even and file is odd = light
            // if rank is odd and file is even = light

            if let Some(piece) = self.get(coord)? {
                let piece_type = piece.piece_type;

                if piece_type == PieceType::PAWN
                    || piece_type == PieceType::ROOK
                    || piece_type == PieceType::QUEEN
                {
                    return Ok(false);
                }

                if self.is_friendly(piece) {
                    if piece_type == PieceType::KNIGHT {
                        friendly_knights += 1;
                    }

                    if piece_type == PieceType::BISHOP {
                        // dark square bishop
                        if coord.is_dark_sq() {
                            friendly_dark_bishops += 1;

                            if enemy_light_bishops >= 1 {
                                return Ok(false);
                            }
                        }

                        // light square bishop
                        if coord.is_light_sq() {
                            friendly_light_bishops += 1;

                            if enemy_dark_bishops >= 1 {
                                return Ok(false);
                            }
                        }

                        friendly_bishops += 1;
                    }
                } else {
                    if piece_type == PieceType::KNIGHT {
                        enemy_knights += 1;
                    }

                    if piece_type == PieceType::BISHOP {
                        // dark square bishop

                        if coord.is_dark_sq() {
                            enemy_dark_bishops += 1;

                            if friendly_light_bishops >= 1 {
                                return Ok(false);
                            }
                        }

                        // light square bishop
                        if coord.is_light_sq() {
                            enemy_light_bishops += 1;

                            if friendly_dark_bishops >= 1 {
                                return Ok(false);
                            }
                        }

                        enemy_bishops += 1;
                    }
                }
            }
        }

        // king vs king
        if friendly_knights == 0
            && friendly_bishops == 0
            && enemy_knights == 0
            && enemy_bishops == 0
        {
            return Ok(true);
        }

        if friendly_knights == 0 && enemy_knights == 0 {
            if friendly_bishops == 2 || enemy_bishops == 2 {
                return Ok(false);
            }

            return Ok(true);
        }

        if friendly_bishops == 0 && enemy_bishops == 0 {
            if friendly_knights >= 1 && enemy_knights >= 1 {
                return Ok(false);
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn is_50_moves(&self) -> ChessResult<bool> {
        Ok(self.half_moves >= 100)
    }

    pub fn is_threefold_repetition(&self) -> ChessResult<bool> {
        let fen = self.get_fen()?;
        let position = fen.split(" ").collect::<Vec<&str>>()[0];

        if let Some(count) = self.unique_positions.get(position) {
            Ok(*count >= 3)
        } else {
            Ok(false)
        }
    }

    /// Check if a square is attacked by opponent pieces
    pub fn is_attacked(&self, from: SquareCoordinate) -> ChessResult<bool> {
        let sliding_attack_deltas = vec![16, -16, 1, -1, 17, 15, -17, -15];
        let knight_attack_deltas = vec![14, 31, 18, 33, -14, -31, -18, -33];

        let from_idx = from.to_index();
        let from_piece = self.get(from)?;

        for delta in sliding_attack_deltas {
            let mut to = from.to_index() as i16 + delta as i16;
            loop {
                if let Ok(_to) = utils::is_valid(to as usize) {
                    let to_sq = (_to as u8).to_coordinate();

                    if let Some(attacker) = self.get(to_sq)? {
                        if attacker.color == self.turn {
                            break;
                        }

                        if let Some(from_piece) = from_piece {
                            if from_piece.color == attacker.color {
                                break;
                            }
                        }

                        let diff = from_idx as i16 - to + 119;
                        let attack_bits_mask = constants::ATTACKS[diff as usize];

                        if attack_bits_mask != 0 {
                            if attacker.piece_type == PieceType::PAWN {
                                let with_color =
                                    attacker.piece_type.to_value() | attacker.color.to_value();

                                if with_color & attack_bits_mask != 0
                                    && attacker.color.to_value() == attack_bits_mask & COLOR_MASK
                                {
                                    return Ok(true);
                                } else {
                                    break;
                                }
                            } else {
                                if (attacker.piece_type.to_value() & attack_bits_mask) != 0 {
                                    return Ok(true);
                                } else {
                                    break;
                                }
                            }
                        }
                    }

                    to += delta as i16;
                } else {
                    break;
                }
            }
        }

        for delta in knight_attack_deltas {
            let to = from.to_index() as i16 + delta as i16;

            if let Ok(_to) = utils::is_valid(to as usize) {
                let to_sq = (_to as u8).to_coordinate();

                if let Some(attacker) = self.get(to_sq)? {
                    if !self.is_friendly(attacker) && attacker.piece_type == PieceType::KNIGHT {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn change_turn(&mut self) {
        if self.turn == Color::WHITE {
            self.turn = Color::BLACK
        } else {
            self.turn = Color::WHITE
        }
    }

    /// Return the PieceType and its associated value on a square. Return an ChessError if the index is out of range.
    ///
    /// Some(t) means an occupied square, None means the square is empty.
    pub fn get(&self, sq: SquareCoordinate) -> ChessResult<Option<Piece>> {
        Ok(self.board.get(sq)?)
    }

    /// Put a piece at specific index on the board. Return the piece and index if succeed, or an ChessError if not.
    ///
    /// Note: this does not update castling rights
    pub fn set(
        &mut self,
        sq: SquareCoordinate,
        piece: PieceType,
        color: Color,
    ) -> ChessResult<(Piece, usize)> {
        let s = self.board.set(sq, piece, color)?;

        if piece == PieceType::KING {
            self.update_kings_positions(sq, color);
        }

        Ok(s)
    }

    pub fn remove(&mut self, sq: SquareCoordinate) -> ChessResult<()> {
        Ok(self.board.remove(sq)?)
    }

    pub fn get_fen(&self) -> ChessResult<String> {
        let mut fen = String::from("");

        let turn = self.turn;
        let en_passant_sq = if let Some(sq) = self.en_passant_sq {
            sq.to_san()
        } else {
            "-".to_string()
        };

        let half_moves = self.half_moves;
        let full_moves = self.full_moves;

        let mut castling_rights = String::new();
        if self.castling_rights.white.kingside {
            castling_rights.push_str("K");
        }

        if self.castling_rights.white.queenside {
            castling_rights.push_str("Q");
        }

        if self.castling_rights.black.kingside {
            castling_rights.push_str("k");
        }

        if self.castling_rights.black.queenside {
            castling_rights.push_str("q")
        }

        if castling_rights.is_empty() {
            castling_rights.push_str("-");
        }

        let mut empty_square: u8 = 0;
        for idx in 0..BOARD_SIZE {
            let idx = match self.board.is_valid(idx) {
                Ok(idx) => idx,
                Err(_) => continue,
            } as u8;

            if let Some(piece) = self.get(idx.to_coordinate())? {
                if empty_square != 0 {
                    fen.push_str(empty_square.to_string().as_str());
                    empty_square = 0;
                }

                if piece.color == Color::WHITE {
                    match piece.piece_type {
                        PieceType::PAWN => fen.push_str("P"),
                        PieceType::ROOK => fen.push_str("R"),
                        PieceType::KNIGHT => fen.push_str("N"),
                        PieceType::BISHOP => fen.push_str("B"),
                        PieceType::QUEEN => fen.push_str("Q"),
                        PieceType::KING => fen.push_str("K"),
                        // _ => panic!("error generating FEN"),
                    }
                } else {
                    match piece.piece_type {
                        PieceType::PAWN => fen.push_str("p"),
                        PieceType::ROOK => fen.push_str("r"),
                        PieceType::KNIGHT => fen.push_str("n"),
                        PieceType::BISHOP => fen.push_str("b"),
                        PieceType::QUEEN => fen.push_str("q"),
                        PieceType::KING => fen.push_str("k"),
                        // _ => panic!("error generating FEN"),
                    }
                }
            } else {
                empty_square += 1;
            }

            if (idx + 1) % 8 == 0 {
                if empty_square != 0 {
                    fen.push_str(empty_square.to_string().as_str());
                    empty_square = 0;
                }

                // if it is the last rank on board, no need to separate with /
                if idx != 119 {
                    fen.push('/');
                }
            }
        }

        Ok(vec![
            fen,
            turn.to_string(),
            castling_rights,
            en_passant_sq,
            half_moves.to_string(),
            full_moves.to_string(),
        ]
        .join(" "))
    }

    pub fn load_fen(&mut self, fen: String) -> ChessResult<()> {
        let fen_parts: Vec<&str> = fen.split(" ").collect();

        let ranks: Vec<&str> = fen_parts[0].split("/").collect();

        let mut idx: usize = 0;
        for rank in ranks {
            for piece in rank.chars() {
                let coord = (BOARD_MAP[idx] as u8).to_coordinate();
                match piece {
                    'p' => {
                        self.set(coord, PieceType::PAWN, Color::BLACK)?;
                    }
                    'r' => {
                        self.set(coord, PieceType::ROOK, Color::BLACK)?;
                    }
                    'n' => {
                        self.set(coord, PieceType::KNIGHT, Color::BLACK)?;
                    }
                    'b' => {
                        self.set(coord, PieceType::BISHOP, Color::BLACK)?;
                    }
                    'q' => {
                        self.set(coord, PieceType::QUEEN, Color::BLACK)?;
                    }
                    'k' => {
                        self.set(coord, PieceType::KING, Color::BLACK)?;
                    }
                    'P' => {
                        self.set(coord, PieceType::PAWN, Color::WHITE)?;
                    }
                    'R' => {
                        self.set(coord, PieceType::ROOK, Color::WHITE)?;
                    }
                    'N' => {
                        self.set(coord, PieceType::KNIGHT, Color::WHITE)?;
                    }
                    'B' => {
                        self.set(coord, PieceType::BISHOP, Color::WHITE)?;
                    }
                    'Q' => {
                        self.set(coord, PieceType::QUEEN, Color::WHITE)?;
                    }
                    'K' => {
                        self.set(coord, PieceType::KING, Color::WHITE)?;
                    }
                    '1'..='8' => idx += piece.to_digit(10).unwrap() as usize - 1,
                    _ => panic!("can't load fen pieces"),
                }

                idx += 1;
            }
        }

        // set turn
        match fen_parts[1] {
            "w" => self.set_turn(Color::WHITE),
            "b" => self.set_turn(Color::BLACK),
            _ => panic!("can't load fen turn"),
        }

        // castling rights
        for castling_right in fen_parts[2].chars() {
            match castling_right {
                'K' => {
                    self.castling_rights.white.kingside = true;
                }
                'Q' => {
                    self.castling_rights.white.queenside = true;
                }
                'k' => {
                    self.castling_rights.black.kingside = true;
                }
                'q' => {
                    self.castling_rights.black.queenside = true;
                }

                '-' => {
                    self.castling_rights.white.kingside = false;
                    self.castling_rights.white.queenside = false;
                    self.castling_rights.black.kingside = false;
                    self.castling_rights.black.kingside = false;
                }
                _ => panic!("cant load fen castling rights"),
            }
        }

        // en passant square
        let square = fen_parts[3];

        match square {
            // no en passant square
            "-" => {
                self.en_passant_sq = None;
            }
            _ => {
                let idx = BOARD_MAP[convert_algebraic_notation_to_index(square) as usize] as u8;
                self.en_passant_sq = Some(idx.to_coordinate());
            }
        }

        self.half_moves = fen_parts[4].parse().unwrap();
        self.full_moves = fen_parts[5].parse().unwrap();

        *self
            .unique_positions
            .entry(fen_parts[0].to_string())
            .or_insert(0) += 1;

        Ok(())
    }

    pub fn perft(&mut self, depth: u8, yah: bool) -> ChessResult<u64> {
        let mut nodes: u64 = 0;
        let mut cnt = Ok(0);

        if depth <= 0 {
            return Ok(1);
        }

        let moves = self.moves()?;

        for _move in moves {
            let from = convert_index_to_algebraic_notation(_move.from.to_index() as u8);
            let to = convert_index_to_algebraic_notation(_move.to.to_index() as u8);

            self.make_move(_move)?;
            self.change_turn();

            cnt = self.perft(depth - 1, false);

            if let Ok(cnt) = cnt {
                nodes += cnt;

                if yah {
                    if let Some(promotion_piece) = _move.promotion_piece {
                        match promotion_piece.piece_type {
                            PieceType::BISHOP => {
                                println!("{} {}", format!("{}{}b", from, to), cnt);
                            }
                            PieceType::KNIGHT => {
                                println!("{} {}", format!("{}{}n", from, to), cnt);
                            }
                            PieceType::ROOK => {
                                println!("{} {}", format!("{}{}r", from, to), cnt);
                            }
                            PieceType::QUEEN => {
                                println!("{} {}", format!("{}{}q", from, to), cnt);
                            }
                            _ => panic!("invalid promotion piece"),
                        }
                    } else {
                        println!("{} {}", format!("{}{}", from, to), cnt);
                    }
                }
            } else {
                panic!(
                    "depth {} mvoe {:?} piece {:?} error {:?}",
                    depth,
                    _move,
                    self.get(_move.from),
                    cnt
                );
            }
            self.undo_move()?;
        }


        return Ok(nodes);
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn set_turn(&mut self, color: Color) {
        self.turn = color;
    }

    fn castling_rights_for_turn(&self) -> (bool, bool) {
        if self.turn == Color::WHITE {
            (
                self.castling_rights.white.kingside,
                self.castling_rights.white.queenside,
            )
        } else {
            (
                self.castling_rights.black.kingside,
                self.castling_rights.black.queenside,
            )
        }
    }

    fn pawn_moves(&self, from: SquareCoordinate) -> ChessResult<Vec<Move>> {
        let mut moves: Vec<Move> = vec![];

        let from_piece = self.get(from)?.ok_or(ChessError::UnknownError(
            "can't generate move for empty piece".to_string(),
        ))?;

        let deltas = match from_piece.color {
            Color::BLACK => BLACK_PAWN_DELTAS,
            Color::WHITE => WHITE_PAWN_DELTAS,
        };

        let mut can_move_forward = true;

        for delta in deltas {
            let to = from.to_index() as i16 + delta as i16;
            if let Ok(_to) = utils::is_valid(to as usize) {
                let to_sq = (_to as u8).to_coordinate();
                let rank = to_sq.rank();

                if delta % 2 != 0 {
                    // normal capture
                    if let Some(attacker) = self.get(to_sq)? {
                        if !self.is_friendly(attacker) {
                            // promotion
                            if rank == 1 || rank == 8 {
                                let color = from_piece.color;
                                let promotion_pieces = vec![
                                    Piece {
                                        piece_type: PieceType::BISHOP,
                                        color,
                                    },
                                    Piece {
                                        piece_type: PieceType::KNIGHT,
                                        color,
                                    },
                                    Piece {
                                        piece_type: PieceType::ROOK,
                                        color,
                                    },
                                    Piece {
                                        piece_type: PieceType::QUEEN,
                                        color,
                                    },
                                ];

                                for piece in promotion_pieces {
                                    moves.push(Move {
                                        from,
                                        to: to_sq,
                                        promotion_piece: Some(piece),
                                    })
                                }
                            } else {
                                moves.push(Move {
                                    from,
                                    to: to_sq,
                                    promotion_piece: None,
                                });
                            }
                        }
                    }

                    // en passant capture
                    if self.en_passant_sq == Some(to_sq) {
                        moves.push(Move {
                            from,
                            to: to_sq,
                            promotion_piece: None,
                        });
                    }
                } else {
                    if self.get(to_sq)?.is_some() {
                        can_move_forward = false;
                    }

                    if !can_move_forward {
                        continue;
                    }

                    let rank = from.rank();
                    // can only do en passant move if rank is 2 for white or 7 for black
                    if to.abs_diff(from.to_index() as i16) == 32 {
                        if from_piece.color == Color::WHITE && rank == 2 {
                            moves.push(Move {
                                from,
                                to: to_sq,
                                promotion_piece: None,
                            });
                        }

                        if from_piece.color == Color::BLACK && rank == 7 {
                            moves.push(Move {
                                from,
                                to: to_sq,
                                promotion_piece: None,
                            });
                        }
                        continue;
                    }

                    let rank = to_sq.rank();

                    if rank == 1 || rank == 8 {
                        let color = from_piece.color;
                        let promotion_pieces = vec![
                            Piece {
                                piece_type: PieceType::BISHOP,
                                color,
                            },
                            Piece {
                                piece_type: PieceType::KNIGHT,
                                color,
                            },
                            Piece {
                                piece_type: PieceType::ROOK,
                                color,
                            },
                            Piece {
                                piece_type: PieceType::QUEEN,
                                color,
                            },
                        ];

                        for piece in promotion_pieces {
                            moves.push(Move {
                                from,
                                to: to_sq,
                                promotion_piece: Some(piece),
                            })
                        }
                    } else {
                        moves.push(Move {
                            from,
                            to: to_sq,
                            promotion_piece: None,
                        })
                    }
                }
            } else {
                continue;
            }
        }

        Ok(moves)
    }

    fn king_moves(&mut self, from: SquareCoordinate) -> ChessResult<Vec<Move>> {
        self.castling_rights.update(&self.kings, &self.board)?;

        let mut moves: Vec<Move> = vec![];

        let deltas = KING_DELTAS.to_vec();
        let (can_castle_kingside, can_castle_queenside) = self.castling_rights_for_turn();
        let from_idx = from.to_index() as i8;

        let is_in_check = self.in_check()?;

        for delta in deltas {
            let to = from.to_index() as i16 + delta as i16;

            if let Ok(_to) = utils::is_valid(to as usize) {
                let to_sq = (_to as u8).to_coordinate();

                let diff = to - from_idx as i16;
                let is_castling = diff == 2 || diff == -2;

                // Kingside castle
                if diff == 2 && !can_castle_kingside {
                    continue;
                }

                // Queenside castle
                if diff == -2 && !can_castle_queenside {
                    continue;
                }

                if let Some(piece) = self.get(to_sq)? {
                    // if we encounter a friendly piece, we can't move there
                    if self.is_friendly(piece) {
                        continue;
                    }
                }

                if is_castling {
                    let mut allow_castle = true;
                    let range: Range<i8> = match diff {
                        2 => Ok(0..2),    // kingside (2 squares to check)
                        -2 => Ok(-4..-1), // queenside (3 squares to check)
                        _ => Err(ChessError::UnknownError("Illegal castle".to_string())),
                    }?;

                    for offset in range {
                        let offset = (offset + 1 + from_idx) as u8;
                        let coord = offset.to_coordinate();

                        // if a piece is blocking the path, we can't castle
                        if self.get(coord)?.is_some() {
                            allow_castle = false;
                            break;
                        }
                        let a = 1;
                        let attacked = self.is_attacked(coord)?;
                        // if king is attacked on the path, we can't castle
                        if attacked
                            && (coord != SquareCoordinate::B1 && coord != SquareCoordinate::B8)
                        {
                            allow_castle = false;
                            break;
                        }
                    }

                    if allow_castle && !is_in_check {
                        moves.push(Move {
                            from,
                            to: to_sq,
                            promotion_piece: None,
                        });
                    }
                } else {
                    moves.push(Move {
                        from,
                        to: to_sq,
                        promotion_piece: None,
                    });
                }
            } else {
                continue;
            }
        }

        Ok(moves)
    }

    fn knight_moves(&self, from: SquareCoordinate) -> ChessResult<Vec<Move>> {
        let mut moves: Vec<Move> = vec![];

        let deltas = KNIGHT_DELTAS.to_vec();

        for delta in deltas {
            let to = from.to_index() as i16 + delta as i16;
            if let Ok(_to) = utils::is_valid(to as usize) {
                let to_sq = (_to as u8).to_coordinate();

                if let Some(piece) = self.get(to_sq)? {
                    // if we encounter a friendly piece, we can't move there
                    if self.is_friendly(piece) {
                        continue;
                    }
                }

                moves.push(Move {
                    from,
                    to: to_sq,
                    promotion_piece: None,
                });
            } else {
                continue;
            }
        }

        Ok(moves)
    }

    fn sliding_moves(&self, from: SquareCoordinate, deltas: Vec<i8>) -> ChessResult<Vec<Move>> {
        let mut moves: Vec<Move> = vec![];

        for delta in deltas {
            let mut to = from.to_index() as i16 + delta as i16;

            loop {
                if let Ok(_to) = utils::is_valid(to as usize) {
                    let to_sq = (_to as u8).to_coordinate();

                    if let Some(piece) = self.get(to_sq)? {
                        // if we encounter a friendly piece, we can't move there
                        if self.is_friendly(piece) {
                            break;
                        } else {
                            // if we encounter an enemy piece, we can capture it but cannot move further
                            moves.push(Move {
                                from,
                                to: to_sq,
                                promotion_piece: None,
                            });
                            break;
                        }
                    } else {
                        moves.push(Move {
                            from,
                            to: to_sq,
                            promotion_piece: None,
                        });
                    }

                    // if the destination square is on the board, we keep searching in that direction until we go off the board
                    to += delta as i16;
                } else {
                    break;
                }
            }
        }

        Ok(moves)
    }

    fn convert_to_internal_move(&self, m: Move) -> ChessResult<InternalMove> {
        let from_piece = self
            .get(m.from)?
            .ok_or(ChessError::InvalidMove(m.from.to_index(), m.to.to_index()))?;

        let to_piece = self.get(m.to)?;

        // create an Internal Move with some defaults
        let mut internal_move = InternalMove {
            move_type: MoveType::Normal,
            from_sq: m.from,
            from_piece,
            to_sq: m.to,
            to_piece,
            promotion_piece: None,
        };

        // capture
        if let Some(to_piece) = to_piece {
            if !self.is_friendly(to_piece) {
                // if the piece isn't friendly, then it is a capture
                internal_move.move_type = MoveType::Capture;
                internal_move.to_piece = Some(to_piece);
            }
        }

        if from_piece.piece_type == PieceType::PAWN {
            // en passant move
            if m.to.to_index().abs_diff(m.from.to_index()) == 32 {
                internal_move.move_type = MoveType::EnPassantMove;
            }

            // en passant capture
            if let Some(en_passant_sq) = self.en_passant_sq {
                if m.to == en_passant_sq {
                    internal_move.move_type = MoveType::EnPassantCapture;
                }
            }

            let rank = m.to.rank();
            // promotion
            if (rank == 8 || rank == 1) {
                internal_move.move_type = MoveType::Promotion;
                internal_move.promotion_piece = m.promotion_piece;
            }
        }

        // castling
        if from_piece.piece_type == PieceType::KING {
            let diff = m.to.to_index() as i8 - m.from.to_index() as i8;
            if diff == 2 {
                internal_move.move_type = MoveType::CastleKingside;
            }

            if diff == -2 {
                internal_move.move_type = MoveType::CastleQueenside;
            }
        }

        Ok(internal_move)
    }

    fn is_friendly(&self, piece: Piece) -> bool {
        self.turn == piece.color
    }

    fn update_kings_positions(&mut self, new_sq: SquareCoordinate, color: Color) {
        if color == Color::WHITE {
            self.kings.white = Some(new_sq)
        } else {
            self.kings.black = Some(new_sq)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PieceType, SquareCoordinate as Square, *};

    #[test]
    fn board_set_and_get_pieces() {
        let mut chess = Chess::new();

        assert_eq!(
            chess.set(Square::__BAD_COORD, PieceType::KING, Color::WHITE),
            Err(ChessError::InvalidIndex(Square::__BAD_COORD.to_index()))
        );

        assert_eq!(chess.get(Square::E1), Ok(None));

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E1, PieceType::KING, Color::WHITE);

        assert_eq!(
            chess.get(Square::E1),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::WHITE
            }))
        );

        let _ = chess.remove(Square::E1);

        assert_eq!(chess.get(Square::E1), Ok(None));
    }

    #[test]
    fn make_move() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E8, PieceType::KING, Color::BLACK);

        assert_eq!(
            chess.get(Square::E8),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::BLACK
            }))
        );

        assert_eq!(
            chess.make_move(Move {
                from: Square::E8,
                to: Square::E7,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(
            chess.make_move(Move {
                from: Square::E3,
                to: Square::E7,
                promotion_piece: None
            }),
            Err(ChessError::InvalidMove(
                Square::E3.to_index(),
                Square::E7.to_index()
            ))
        );
    }

    #[test]
    fn en_passant_move() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E2, PieceType::PAWN, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E2,
                to: Square::E4,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.en_passant_sq, Some(Square::E3));

        assert_eq!(
            chess.make_move(Move {
                from: Square::E4,
                to: Square::E5,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.en_passant_sq, None);
    }

    #[test]
    fn en_passant_capture() {
        let mut chess = Chess::new();

        let _ = chess.set(Square::E5, PieceType::PAWN, Color::WHITE);
        let _ = chess.set(Square::D5, PieceType::PAWN, Color::BLACK);

        chess.en_passant_sq = Some(Square::D6);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E5,
                to: Square::D6,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.get(Square::D5), Ok(None));
        assert_eq!(chess.en_passant_sq, None);
        assert_eq!(
            chess.white_captures,
            vec![Piece {
                piece_type: PieceType::PAWN,
                color: Color::BLACK
            }]
        )
    }

    #[test]
    fn undo_normal_move() {
        let mut chess = Chess::new();

        let _ = chess.set(Square::E1, PieceType::KING, Color::WHITE);

        let _ = chess.make_move(Move {
            from: Square::E1,
            to: Square::E2,
            promotion_piece: None,
        });

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(chess.get(Square::E2), Ok(None));

        assert_eq!(
            chess.get(Square::E1),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::WHITE,
            }))
        );
    }

    #[test]
    fn undo_en_passant_capture() {
        let mut chess = Chess::new();

        let _ = chess.set(Square::E5, PieceType::PAWN, Color::WHITE);
        let _ = chess.set(Square::D5, PieceType::PAWN, Color::BLACK);

        chess.en_passant_sq = Some(Square::D6);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E5,
                to: Square::D6,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(chess.get(Square::D6), Ok(None));
        assert_eq!(
            chess.get(Square::D5),
            Ok(Some(Piece {
                piece_type: PieceType::PAWN,
                color: Color::BLACK
            }))
        );
        assert_eq!(
            chess.get(Square::E5),
            Ok(Some(Piece {
                piece_type: PieceType::PAWN,
                color: Color::WHITE
            }))
        );
    }

    #[test]
    fn undo_en_passant_move() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E2, PieceType::PAWN, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E2,
                to: Square::E4,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.en_passant_sq, Some(Square::E3));

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(chess.get(Square::E4), Ok(None));

        assert_eq!(
            chess.get(Square::E2),
            Ok(Some(Piece {
                piece_type: PieceType::PAWN,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.en_passant_sq, None);
    }

    #[test]
    fn undo_capture() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::F1, PieceType::BISHOP, Color::WHITE);
        let _ = chess.set(Square::C4, PieceType::QUEEN, Color::BLACK);

        assert_eq!(
            chess.make_move(Move {
                from: Square::F1,
                to: Square::C4,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(
            chess.white_captures,
            vec![Piece {
                piece_type: PieceType::QUEEN,
                color: Color::BLACK
            }]
        );

        assert_eq!(
            chess.get(Square::C4),
            Ok(Some(Piece {
                piece_type: PieceType::BISHOP,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(chess.white_captures, vec![]);

        assert_eq!(
            chess.get(Square::F1),
            Ok(Some(Piece {
                piece_type: PieceType::BISHOP,
                color: Color::WHITE
            }))
        );

        assert_eq!(
            chess.get(Square::C4),
            Ok(Some(Piece {
                piece_type: PieceType::QUEEN,
                color: Color::BLACK
            }))
        );
    }

    #[test]
    fn undo_kingside_castle() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E1, PieceType::KING, Color::WHITE);
        let _ = chess.set(Square::H1, PieceType::ROOK, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E1,
                to: Square::G1,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(
            chess.get(Square::E1),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::WHITE
            }))
        );

        assert_eq!(
            chess.get(Square::H1),
            Ok(Some(Piece {
                piece_type: PieceType::ROOK,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.get(Square::G1), Ok(None));
        assert_eq!(chess.get(Square::F1), Ok(None));
        assert_eq!(chess.castling_rights.white.kingside, true);
        assert_eq!(chess.castling_rights.white.queenside, true);
    }

    #[test]
    fn undo_queenside_castle() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E8, PieceType::KING, Color::BLACK);
        let _ = chess.set(Square::A8, PieceType::ROOK, Color::BLACK);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E8,
                to: Square::C8,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(
            chess.get(Square::E8),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::BLACK
            }))
        );

        assert_eq!(
            chess.get(Square::A8),
            Ok(Some(Piece {
                piece_type: PieceType::ROOK,
                color: Color::BLACK
            }))
        );

        assert_eq!(chess.get(Square::C8), Ok(None));
        assert_eq!(chess.get(Square::D8), Ok(None));
        assert_eq!(chess.castling_rights.black.kingside, true);
        assert_eq!(chess.castling_rights.black.queenside, true);
    }

    #[test]
    fn undo_kingside_promotion() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E7, PieceType::PAWN, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E7,
                to: Square::E8,
                promotion_piece: Some(Piece {
                    piece_type: PieceType::QUEEN,
                    color: Color::WHITE
                })
            }),
            Ok(())
        );

        assert_eq!(chess.undo_move(), Ok(()));

        assert_eq!(
            chess.get(Square::E7),
            Ok(Some(Piece {
                piece_type: PieceType::PAWN,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.get(Square::E8), Ok(None));
    }

    #[test]
    fn castle_kingside_successfully() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E1, PieceType::KING, Color::WHITE);
        let _ = chess.set(Square::H1, PieceType::ROOK, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E1,
                to: Square::G1,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.get(Square::H1), Ok(None));

        assert_eq!(
            chess.get(Square::G1),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::WHITE
            }))
        );

        assert_eq!(
            chess.get(Square::F1),
            Ok(Some(Piece {
                piece_type: PieceType::ROOK,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.castling_rights.white.kingside, false);
        assert_eq!(chess.castling_rights.white.queenside, false);
    }

    #[test]
    fn castle_queenside_successfully() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E8, PieceType::KING, Color::BLACK);
        let _ = chess.set(Square::A8, PieceType::ROOK, Color::BLACK);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E8,
                to: Square::C8,
                promotion_piece: None
            }),
            Ok(())
        );

        assert_eq!(chess.get(Square::A8), Ok(None));

        assert_eq!(
            chess.get(Square::C8),
            Ok(Some(Piece {
                piece_type: PieceType::KING,
                color: Color::BLACK
            }))
        );

        assert_eq!(
            chess.get(Square::D8),
            Ok(Some(Piece {
                piece_type: PieceType::ROOK,
                color: Color::BLACK
            }))
        );

        assert_eq!(chess.castling_rights.black.queenside, false);
        assert_eq!(chess.castling_rights.black.kingside, false);
    }

    #[test]
    fn cannot_castle_if_rook_not_in_correct_square() {
        let mut chess = Chess::new();

        let _ = chess.set(Square::E1, PieceType::KING, Color::WHITE);
        let _ = chess.set(Square::H1, PieceType::ROOK, Color::WHITE);
        let _ = chess.set(Square::A1, PieceType::ROOK, Color::WHITE);

        let _ = chess.make_move(Move {
            from: Square::H1,
            to: Square::H2,
            promotion_piece: None,
        });

        assert_eq!(chess.castling_rights.white.queenside, true);
        assert_eq!(chess.castling_rights.white.kingside, false);

        let _ = chess.make_move(Move {
            from: Square::A1,
            to: Square::D1,
            promotion_piece: None,
        });

        assert_eq!(chess.castling_rights.white.queenside, false);
        assert_eq!(chess.castling_rights.white.kingside, false);
    }

    #[test]
    fn promotion() {
        let mut chess = Chess::new();

        // assigning to empty var to ignore warning
        let _ = chess.set(Square::E7, PieceType::PAWN, Color::WHITE);

        assert_eq!(
            chess.make_move(Move {
                from: Square::E7,
                to: Square::E8,
                promotion_piece: Some(Piece {
                    piece_type: PieceType::QUEEN,
                    color: Color::WHITE
                })
            }),
            Ok(())
        );

        assert_eq!(
            chess.get(Square::E8),
            Ok(Some(Piece {
                piece_type: PieceType::QUEEN,
                color: Color::WHITE
            }))
        );

        assert_eq!(chess.get(Square::E7), Ok(None))
    }
}
