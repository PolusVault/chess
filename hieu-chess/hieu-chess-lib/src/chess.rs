use crate::board::{Board, SIZE_0X88};
use crate::constants::{ATTACKS, COLOR_MASK, KNIGHT_DELTAS, QUEEN_DELTAS};
use crate::error::Error;
use crate::move_gen::MoveGen;
use crate::piece::{PType, Piece};
use crate::square::Square;
use crate::utils;
use std::cmp;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Color {
    WHITE = 0,
    BLACK = 128,
}

impl TryFrom<&str> for Color {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "w" => Ok(Color::WHITE),
            "b" => Ok(Color::BLACK),
            _ => Err(Error::InvalidColor),
        }
    }
}

impl From<Color> for &'static str {
    fn from(color: Color) -> &'static str {
        match color {
            Color::WHITE => "w",
            Color::BLACK => "b",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion_piece: Option<Piece>,
}

// TODO: implement algebraic notation for moves
impl Move {
    pub fn new(from: Square, to: Square, promotion_piece: Option<Piece>) -> Self {
        Self {
            from,
            to,
            promotion_piece,
        }
    }

    pub fn from_str(from: &str, to: &str, promotion_piece: Option<Piece>) -> Self {
        Self {
            from: from.try_into().unwrap(),
            to: to.try_into().unwrap(),
            promotion_piece,
        }
    }
}

#[derive(Default, Clone)]
struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

struct HistoryEntry {
    m: Move,
    piece: Piece,
    castling_rights: CastlingRights,
    en_passant_sq: Option<Square>,
    half_moves: u8,
    full_moves: u8,
    is_queenside_castle: bool,
    is_kingside_castle: bool,
    has_moved: bool,
    unique_positions: HashMap<String, u8>,
    side_to_move: Color,
    fen: String,
    is_capture: bool,
    check_rays: HashSet<Square>,
}

pub struct Capture {
    sq: Square,
    piece: Piece,
}

pub struct GameState {
    pub board: Board,
    pub en_passant_sq: Option<Square>,
    pub side_to_move: Color,
    pub is_in_check: bool,
    pub white_king_square: Option<Square>,
    pub black_king_square: Option<Square>,
    check_rays: HashSet<Square>,
    castling_rights: CastlingRights,

    pub captures: Vec<Capture>,
    unique_positions: HashMap<String, u8>,

    half_moves: u8,
    full_moves: u8,
    debug: bool,

    history: Vec<HistoryEntry>,

    has_moved: bool,
    captures_count: usize,
    castle_count: usize,
    check_count: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            side_to_move: Color::WHITE,
            board: Board::new(),
            en_passant_sq: None,
            is_in_check: false,
            check_rays: HashSet::new(),
            white_king_square: None,
            black_king_square: None,
            castling_rights: CastlingRights::default(),
            captures: vec![],
            debug: false,
            full_moves: 0,
            half_moves: 0,
            unique_positions: HashMap::new(),
            has_moved: false,
            captures_count: 0,
            castle_count: 0,
            check_count: 0,

            history: vec![],
        }
    }

    pub fn play_move(&mut self, m: Move) -> Result<(), Error> {
        if !self.is_occupied(&m.from) {
            return Err(Error::UnknownMove);
        }

        if self
            .board
            .get(&m.from)
            .expect("piece must be present after the is_occupied check")
            .color
            != self.side_to_move
        {
            return Err(Error::MustWaitForTurn);
        }

        // TODO remove this later
        let legal_moves = MoveGen::moves_for_square(self, m.from);
        if !legal_moves.contains(&m) || legal_moves.len() == 0 {
            return Err(Error::IllegalMove);
        }

        self.make_move(m);
        self.change_turn();
        self.update_king_attacks();

        Ok(())
    }

    // this method assumes all moves are valid, and there must be a piece being moved
    pub fn make_move(&mut self, m: Move) {
        let piece = self
            .board
            .get(&m.from)
            .expect("a piece must be present in make_move")
            .clone();

        let mut is_capture = false;
        if self.is_capture(&m) {
            if self.is_enpassant_capture(&m) {
                let delta: i8 = match piece.color {
                    Color::WHITE => -16,
                    Color::BLACK => 16,
                };
                let to =
                    m.to.add(delta)
                        .expect("en passant square must be valid here");

                self.board.remove(&to);

                self.captures.push(Capture {
                    sq: to,
                    piece: Piece::new(
                        PType::PAWN,
                        if piece.color == Color::WHITE {
                            Color::BLACK
                        } else {
                            Color::WHITE
                        },
                    ),
                });
            } else {
                self.captures.push(Capture {
                    sq: m.to.clone(),
                    piece: self
                        .board
                        .get(&m.to)
                        .expect("capture must have a captured piece")
                        .clone(),
                });
            }
            is_capture = true;
        }

        // TODO: refactor this
        let mut is_queenside_castle = false;
        let mut is_kingside_castle = false;
        if self.board.get(&m.from).unwrap().p_type == PType::KING {
            if self.is_castling_kingside(&m) {
                is_kingside_castle = true;
                // remove the rook
                self.board
                    .remove(&m.to.add(1).expect("castling square must be valid"));
                // put a new rook to the left
                self.set(
                    Piece::new(PType::ROOK, piece.color),
                    &m.to.add(-1).expect("castling square must be valid"),
                );
                self.castle_count += 1;
            } else if self.is_castling_queenside(&m) {
                is_queenside_castle = true;
                // remove the rook
                self.board
                    .remove(&m.to.add(-2).expect("castling square must be valid"));
                // put a new rook to the right
                self.set(
                    Piece::new(PType::ROOK, piece.color),
                    &m.to.add(1).expect("castling square must be valid"),
                );
                self.castle_count += 1;
            }
        }

        // TODO: we can do better
        self.history.push(HistoryEntry {
            m: m.clone(),
            piece: piece.clone(),
            castling_rights: self.castling_rights.clone(),
            full_moves: self.full_moves,
            half_moves: self.half_moves,
            en_passant_sq: self.en_passant_sq,
            is_kingside_castle,
            is_queenside_castle,
            has_moved: self.has_moved,
            unique_positions: self.unique_positions.clone(),
            side_to_move: self.side_to_move,
            fen: self.get_fen(),
            is_capture,
            check_rays: self.check_rays.clone(),
        });

        if self.is_enpassant_move(&m) {
            let delta: i8 = match piece.color {
                Color::WHITE => -16,
                Color::BLACK => 16,
            };

            self.en_passant_sq = Some(m.to.add(delta).expect("en passant square must be valid"));
        } else {
            self.en_passant_sq = None;
        }

        self.has_moved = true;
        self.full_moves += 1;

        if piece.p_type == PType::PAWN || self.is_capture(&m) {
            self.half_moves = 0;
        } else {
            self.half_moves += 1;
        }

        if let Some(ref promo_piece) = m.promotion_piece {
            self.set(promo_piece.clone(), &m.to);
        } else {
            self.set(piece.clone(), &m.to);
        }

        self.board.remove(&m.from);
        self.update_castling_rights(&piece, &m);
        self.update_positions();
    }

    pub fn undo(&mut self) {
        if let Some(entry) = self.history.pop() {
            let piece = entry.piece;

            self.set(piece.clone(), &entry.m.from);
            self.board.remove(&entry.m.to);

            if entry.is_kingside_castle {
                // remove the rook
                self.board.remove(&entry.m.from.add(1).unwrap());
                self.set(
                    Piece::new(PType::ROOK, piece.color),
                    &entry.m.from.add(3).unwrap(),
                );
            } else if entry.is_queenside_castle {
                self.board.remove(&entry.m.from.add(-1).unwrap());
                self.set(
                    Piece::new(PType::ROOK, piece.color),
                    &entry.m.from.add(-4).unwrap(),
                );
            }

            self.full_moves = entry.full_moves;
            self.half_moves = entry.half_moves;
            self.has_moved = entry.has_moved;
            self.unique_positions = entry.unique_positions;
            self.side_to_move = entry.side_to_move;
            self.castling_rights = entry.castling_rights;
            self.en_passant_sq = entry.en_passant_sq;
            self.check_rays = entry.check_rays;
            self.is_in_check = self.check_rays.len() > 0;

            if entry.is_capture {
                let c = self.captures.pop().expect("capture MUST be in history");
                self.board.set(c.piece, &c.sq);
            }
        };
    }

    fn update_king_attacks(&mut self) {
        let king_sq: Square = self.get_current_king_sq().expect("no king to check");

        // *self.check_count.borrow_mut() = 0;
        self.check_rays = self.get_attack_rays(king_sq);
        self.is_in_check = self.check_rays.len() > 0;
    }

    fn get_attack_rays(&self, sq: Square) -> HashSet<Square> {
        #[allow(non_snake_case)]
        let SLIDING_ATTACK_DELTAS = QUEEN_DELTAS;
        let mut temp: Square;
        let mut attack_rays: HashSet<Square> = HashSet::new();

        for delta in SLIDING_ATTACK_DELTAS {
            let mut attacker_present = false;
            let mut visting_squares: HashSet<Square> = HashSet::new();
            temp = sq;

            while let Ok(to_sq) = temp.add(*delta) {
                let Some(piece) = self.board.get(&to_sq) else {
                    temp = to_sq;
                    visting_squares.insert(to_sq);
                    continue;
                };

                if piece.color == self.side_to_move {
                    break; // stop looking in this direction if a friendly piece is blocking
                }

                let diff = to_sq.0 as i16 - sq.0 as i16 + 119;
                let attack_mask = ATTACKS[diff as usize];

                if attack_mask == 0 {
                    break; // this piece can't attack that direction, so we stop looking that direction
                }

                if piece.p_type == PType::PAWN {
                    let piece_bits = (piece.p_type as u8) | (piece.color as u8);

                    if (piece_bits & attack_mask) != 0
                        && (piece.color as u8) == (attack_mask & COLOR_MASK)
                    {
                        attacker_present = true;
                        visting_squares.insert(to_sq);
                    }
                } else {
                    if ((piece.p_type as u8) & attack_mask) != 0 {
                        attacker_present = true;
                        visting_squares.insert(to_sq);
                    }
                }

                break;
            }

            if attacker_present {
                attack_rays.extend(&visting_squares);
            }
        }

        // check for knight attacks
        for delta in KNIGHT_DELTAS {
            if let Ok(to_sq) = sq.add(*delta) {
                let Some(piece) = self.board.get(&to_sq) else {
                    continue;
                };

                if piece.p_type == PType::KNIGHT && piece.color != self.side_to_move {
                    attack_rays.insert(to_sq);
                }
            }
        }

        return attack_rays;
    }

    pub fn is_attacked(&self, sq: Square) -> bool {
        return self.get_attack_rays(sq).len() > 0;
    }

    pub fn get_current_king_sq(&self) -> Option<Square> {
        if self.side_to_move == Color::WHITE {
            return self.white_king_square;
        } else {
            return self.black_king_square;
        }
    }

    pub fn get_castling_rights(&self) -> (bool, bool) {
        if self.side_to_move == Color::WHITE {
            return (
                self.castling_rights.white_kingside,
                self.castling_rights.white_queenside,
            );
        } else {
            return (
                self.castling_rights.black_kingside,
                self.castling_rights.black_queenside,
            );
        }
    }

    fn update_castling_rights(&mut self, piece: &Piece, m: &Move) {
        let a1: Square = "a1".try_into().unwrap();
        let h1: Square = "h1".try_into().unwrap();
        let a8: Square = "a8".try_into().unwrap();
        let h8: Square = "h8".try_into().unwrap();

        if self.board.get(&a1) != Some(&Piece::new(PType::ROOK, Color::WHITE)) {
            self.castling_rights.white_queenside = false;
        }
        if self.board.get(&h1) != Some(&Piece::new(PType::ROOK, Color::WHITE)) {
            self.castling_rights.white_kingside = false;
        }
        if self.board.get(&a8) != Some(&Piece::new(PType::ROOK, Color::BLACK)) {
            self.castling_rights.black_queenside = false;
        }
        if self.board.get(&h8) != Some(&Piece::new(PType::ROOK, Color::BLACK)) {
            self.castling_rights.black_kingside = false;
        }

        if piece.p_type == PType::KING {
            if piece.color == Color::WHITE {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }

            return;
        }

        if piece.p_type != PType::ROOK {
            return;
        }

        let kingside_rook_sq = match piece.color {
            Color::WHITE => "h1",
            Color::BLACK => "h8",
        };

        let queenside_rook_sq = match piece.color {
            Color::WHITE => "a1",
            Color::BLACK => "a8",
        };

        if m.from.get_notation() == kingside_rook_sq {
            if piece.color == Color::WHITE {
                self.castling_rights.white_kingside = false;
            } else {
                self.castling_rights.black_kingside = false;
            }
        } else if m.from.get_notation() == queenside_rook_sq {
            if piece.color == Color::WHITE {
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_queenside = false;
            }
        }
    }

    fn update_king_sq(&mut self, sq: Square, color: Color) {
        if color == Color::WHITE {
            self.white_king_square = Some(sq);
        } else {
            self.black_king_square = Some(sq);
        }
    }

    fn update_positions(&mut self) {
        let fen = self.get_fen();
        let fen_parts: Vec<&str> = fen.split(" ").collect();

        *self
            .unique_positions
            .entry(fen_parts[0].to_string())
            .or_insert(0) += 1;
    }

    fn change_turn(&mut self) {
        if self.side_to_move == Color::WHITE {
            self.side_to_move = Color::BLACK;
        } else {
            self.side_to_move = Color::WHITE;
        }
    }

    pub fn reset(&mut self) {
        self.side_to_move = Color::WHITE;
        self.board = Board::new();
        self.en_passant_sq = None;
        self.is_in_check = false;
        self.check_rays = HashSet::new();
        self.white_king_square = None;
        self.black_king_square = None;
        self.castling_rights = CastlingRights::default();
        self.captures = vec![];
        self.debug = false;
        self.full_moves = 0;
        self.half_moves = 0;
        self.unique_positions = HashMap::new();
        self.has_moved = false;
    }

    fn is_draw(&mut self) -> bool {
        self.is_stalemate()
            || self.is_threefold_repetition()
            || self.is_50_moves()
            || self.is_insufficient_material()
    }

    fn is_stalemate(&mut self) -> bool {
        !self.is_in_check && MoveGen::moves(self, self.side_to_move).len() == 0
    }

    // https://www.chess.com/article/view/how-chess-games-can-end-8-ways-explained#insufficient-material
    // TODO: i think we can do better
    fn is_insufficient_material(&self) -> bool {
        let mut w_knights = 0;
        let mut b_knights = 0;
        let mut w_bishops = 0;
        let mut b_bishops = 0;
        let mut w_light_square_bishops = 0;
        let mut w_dark_square_bishops = 0;
        let mut b_light_square_bishops = 0;
        let mut b_dark_square_bishops = 0;
        let mut knights = 0;
        let mut bishops = 0;

        for idx in 0..SIZE_0X88 {
            let idx = idx as u8;

            if utils::is_valid_idx(idx).is_err() {
                continue;
            }

            let Some(piece) = self.board.get(&idx) else {
                continue;
            };

            if piece.p_type == PType::PAWN
                || piece.p_type == PType::ROOK
                || piece.p_type == PType::QUEEN
            {
                return false;
            }

            let square: Square = idx.try_into().unwrap();

            if piece.color == Color::WHITE {
                if piece.p_type == PType::KNIGHT {
                    w_knights += 1;
                    knights += 1;
                }

                if piece.p_type == PType::BISHOP {
                    if square.color() == Color::WHITE {
                        w_light_square_bishops += 1;
                    } else {
                        w_dark_square_bishops += 1;
                    }

                    w_bishops += 1;
                    bishops += 1;
                }
            } else {
                if piece.p_type == PType::KNIGHT {
                    b_knights += 1;
                    knights += 1;
                }

                if piece.p_type == PType::BISHOP {
                    if square.color() == Color::WHITE {
                        b_light_square_bishops += 1;
                    } else {
                        b_dark_square_bishops += 1;
                    }

                    b_bishops += 1;
                    bishops += 1;
                }
            }
        }

        // king vs king
        if knights == 0 && bishops == 0 {
            return true;
        }

        // king + minor piece vs king
        if (knights == 1 && bishops == 0) || (bishops == 1 && knights == 0) {
            return true;
        }

        // king + two knights vs king
        // if (w_knights == 2 && bishops == 0) || (b_knights == 2 && bishops == 0) {
        //     return true;
        // }

        // king + minor piece vs king + minor piece (minor piece is different kind)
        // if (w_knights == 1 && b_bishops == 1) || (w_bishops == 1 && b_knights == 1) {
        //     return true;
        // }

        // king + bishop vs king + bishop (same color bishops)
        if (w_dark_square_bishops == 1 && b_dark_square_bishops == 1)
            || (w_light_square_bishops == 1 && b_light_square_bishops == 1)
        {
            return true;
        }

        if w_dark_square_bishops != 0 && b_dark_square_bishops != 0 && knights == 0 {
            return true;
        }

        if w_light_square_bishops != 0 && b_light_square_bishops != 0 && knights == 0 {
            return true;
        }

        false
    }

    fn is_50_moves(&self) -> bool {
        self.half_moves >= 100
    }

    // this may not work as intended
    fn is_threefold_repetition(&self) -> bool {
        for (_, position_count) in &self.unique_positions {
            if *position_count >= 3 {
                return true;
            }
        }

        false
    }

    fn is_checkmate(&mut self) -> bool {
        self.is_in_check && MoveGen::moves(self, self.side_to_move).len() == 0
    }

    pub fn is_occupied(&self, sq: &Square) -> bool {
        return self.board.get(sq).is_some();
    }

    fn is_friendly(&self, piece: &Piece) -> bool {
        return piece.color == self.side_to_move;
    }

    fn is_enpassant_capture(&self, m: &Move) -> bool {
        self.board.get(&m.from).unwrap().p_type == PType::PAWN && Some(m.to) == self.en_passant_sq
    }

    fn is_enpassant_move(&self, m: &Move) -> bool {
        let p = self.board.get(&m.from).unwrap();

        if p.p_type != PType::PAWN {
            return false;
        }

        if (m.to.0 as i16 - m.from.0 as i16).abs() != 32 {
            return false;
        }

        for d in &[1, -1] {
            if let Ok(to) = m.to.add(*d) {
                if let Some(piece) = self.board.get(&to) {
                    if piece.p_type == PType::PAWN && piece.color != p.color {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    fn is_capture(&self, m: &Move) -> bool {
        let Some(p) = self.board.get(&m.to) else {
            let a = self.is_enpassant_capture(m);
            return a;
        };
        p.color != self.side_to_move
    }

    pub fn is_castling(&self, m: &Move) -> bool {
        (m.to.0 as i16 - m.from.0 as i16).abs() == 2
        // let Some(king) = self.board.get(&m.from) else {
        //     return false;
        // };

        // if king.color == Color::WHITE {
        //     return m.from == "e1".try_into().unwrap()
        //         && (m.to == "g1".try_into().unwrap() || m.to == "c1".try_into().unwrap());
        // } else {
        //     return m.from == "e8".try_into().unwrap()
        //         && (m.to == "g8".try_into().unwrap() || m.to == "c8".try_into().unwrap());
        // }
    }

    pub fn is_castling_kingside(&self, m: &Move) -> bool {
        let a = m.from.get_notation();
        (a == "e1" || a == "e8") && (m.to.0 as i16 - m.from.0 as i16) == 2
    }

    pub fn is_castling_queenside(&self, m: &Move) -> bool {
        let a = m.from.get_notation();
        (a == "e1" || a == "e8") && (m.to.0 as i16 - m.from.0 as i16) == -2
    }

    fn set(&mut self, piece: Piece, sq: &Square) {
        if piece.p_type == PType::KING {
            self.update_king_sq(sq.clone(), piece.color);
        }
        self.board.set(piece, sq);
    }

    // TODO: handle error while parsing FEN, full moves
    pub fn load_fen(&mut self, fen: &str) {
        let fen_parts: Vec<&str> = fen.split(" ").collect();

        let mut ranks: Vec<&str> = fen_parts[0].split("/").collect();
        ranks.reverse();

        for rank_idx in 0..ranks.len() {
            let mut file_idx: u8 = 0;

            for c in ranks[rank_idx].chars() {
                let sq = Square::new((rank_idx as u8).into(), file_idx.into());

                match c {
                    '1'..='8' => {
                        file_idx += (c.to_digit(10).unwrap() as usize - 1) as u8;
                    }
                    _ => {
                        self.set(
                            c.to_string()
                                .as_str()
                                .try_into()
                                .expect("invalid FEN characters"),
                            &sq,
                        );
                    }
                };

                file_idx += 1;
            }
        }

        //set turn
        match fen_parts[1] {
            "w" => self.side_to_move = Color::WHITE,
            "b" => self.side_to_move = Color::BLACK,
            _ => panic!("invalid FEN turn"),
        }

        //castling rights
        for castling_right in fen_parts[2].chars() {
            match castling_right {
                'K' => {
                    self.castling_rights.white_kingside = true;
                }
                'Q' => {
                    self.castling_rights.white_queenside = true;
                }
                'k' => {
                    self.castling_rights.black_kingside = true;
                }
                'q' => {
                    self.castling_rights.black_queenside = true;
                }

                '-' => {
                    self.castling_rights.white_kingside = false;
                    self.castling_rights.white_queenside = false;
                    self.castling_rights.black_kingside = false;
                    self.castling_rights.black_kingside = false;
                }
                _ => panic!("cant load fen castling rights"),
            }
        }

        // en passant square
        let en_passant_square = fen_parts[3];

        match en_passant_square {
            "-" => {
                self.en_passant_sq = None;
            }
            _ => {
                self.en_passant_sq = Some(
                    en_passant_square
                        .try_into()
                        .expect("invalid en passant square in FEN"),
                );
            }
        }

        self.half_moves = fen_parts[4].parse().expect("can't parse FEN half moves");
        self.full_moves = fen_parts[5].parse().expect("can't parse FEN full moves");

        *self
            .unique_positions
            .entry(fen_parts[0].to_string())
            .or_insert(0) += 1;

        self.update_king_attacks();
    }

    // TODO refactor this
    pub fn get_fen(&self) -> String {
        let mut empty_count: u8 = 0;

        let mut rank = String::new();
        let mut ranks: Vec<String> = vec![];

        for idx in 0..128 {
            if utils::is_valid_idx(idx).is_err() {
                continue;
            }

            if let Some(piece) = self.board.get(&idx) {
                if empty_count != 0 {
                    rank.push_str(empty_count.to_string().as_str());
                    empty_count = 0;
                }

                let p: String = piece.clone().into();
                rank.push_str(&p);
            } else {
                empty_count += 1;
            }

            if (idx + 1) % 8 == 0 {
                if empty_count != 0 {
                    rank.push_str(empty_count.to_string().as_str());
                    empty_count = 0;
                }

                ranks.push(rank);
                rank = String::new();
            }
        }

        ranks.reverse();
        let en_passant_sq = if let Some(sq) = self.en_passant_sq {
            sq.get_notation()
        } else {
            "-".to_string()
        };

        let half_moves = self.half_moves;
        let full_moves = if self.has_moved {
            cmp::max(1, self.full_moves - 1)
        } else {
            self.full_moves
        };

        let mut castling_rights = String::new();
        if self.castling_rights.white_kingside {
            castling_rights.push_str("K");
        }

        if self.castling_rights.white_queenside {
            castling_rights.push_str("Q");
        }

        if self.castling_rights.black_kingside {
            castling_rights.push_str("k");
        }

        if self.castling_rights.black_queenside {
            castling_rights.push_str("q")
        }

        if castling_rights.is_empty() {
            castling_rights.push_str("-");
        }

        let turn: &str = self.side_to_move.into();

        // TODO: use format!
        vec![
            ranks.join("/"),
            turn.to_string(),
            castling_rights,
            en_passant_sq,
            half_moves.to_string(),
            full_moves.to_string(),
        ]
        .join(" ")
    }

    pub fn get_boar(&self) -> Board {
        self.board.clone()
    }

    pub fn perft(&mut self, depth: u8, log: bool) -> usize {
        let mut nodes = 0;
        let mut count = 0;

        if depth <= 0 {
            return 1;
        }

        let moves = MoveGen::moves(self, self.side_to_move);

        for _move in moves {
            let from = _move.from.get_notation();
            let to = _move.to.get_notation();

            match self.play_move(_move.clone()) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e)
                }
            }
            // if self.is_capture(&_move) && depth == 1 {
            //     self.captures_count += 1;
            // }
            // if self.is_in_check && depth == 1 {
            //     self.check_count += 1;
            // }
            count = self.perft(depth - 1, false);

            nodes += count;

            if log {
                if let Some(promotion_piece) = _move.promotion_piece {
                    match promotion_piece.p_type {
                        PType::BISHOP => {
                            println!("{} {}", format!("{}{}b", from, to), count);
                        }
                        PType::KNIGHT => {
                            println!("{} {}", format!("{}{}n", from, to), count);
                        }
                        PType::ROOK => {
                            println!("{} {}", format!("{}{}r", from, to), count);
                        }
                        PType::QUEEN => {
                            println!("{} {}", format!("{}{}q", from, to), count);
                        }
                        _ => panic!("invalid promotion piece"),
                    }
                } else {
                    println!("{} {}", format!("{}{}", from, to), count);
                }
            }

            self.undo();
        }

        return nodes;
    }
}

pub struct Chess {
    state: GameState,
}

impl Chess {
    pub fn new() -> Self {
        let default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut state = GameState::new();
        state.load_fen(default_fen);

        Self { state }
    }

    pub fn play_move(&mut self, m: Move) -> Result<(), Error> {
        Ok(self.state.play_move(m)?)
    }

    pub fn moves_for_square(&mut self, sq: Square) -> Vec<Move> {
        MoveGen::moves_for_square(&mut self.state, sq)
    }

    pub fn is_draw(&mut self) -> bool {
        self.state.is_draw()
    }

    pub fn is_stalemate(&mut self) -> bool {
        self.state.is_stalemate()
    }

    pub fn is_threefold_repetition(&mut self) -> bool {
        self.state.is_threefold_repetition()
    }

    pub fn is_50_moves(&self) -> bool {
        self.state.is_50_moves()
    }

    pub fn is_insufficient_material(&self) -> bool {
        self.state.is_insufficient_material()
    }

    pub fn is_checkmate(&mut self) -> bool {
        self.state.is_checkmate()
    }

    pub fn load_fen(&mut self, fen: &str) -> Result<(), Error> {
        self.state.load_fen(fen);

        Ok(())
    }

    pub fn get_fen(&self) -> String {
        self.state.get_fen()
    }

    pub fn get_turn(&self) -> Color {
        self.state.side_to_move
    }

    pub fn set_turn(&mut self, color: Color) {
        self.state.side_to_move = color;
    }

    pub fn get_board_ptr(&self) -> *const Option<Piece> {
        self.state.board.get_board_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::*;
    use crate::square::*;

    #[test]
    fn in_check() {
        let mut state = GameState::new();
        let white_king_sq = Square::new(Rank::Five, File::C);
        let attacker_sq = Square::new(Rank::Three, File::E);
        let attacker2_sq = Square::new(Rank::Eight, File::F);

        let king = Piece::new(PType::KING, Color::WHITE);
        let attacker = Piece::new(PType::QUEEN, Color::BLACK);
        let attacker2 = Piece::new(PType::BISHOP, Color::BLACK);

        state.set(king, &white_king_sq);
        state.set(attacker, &attacker_sq);
        state.set(attacker2, &attacker2_sq);
        state.set(
            Piece::new(PType::PAWN, Color::BLACK),
            &Square::new(Rank::Six, File::B),
        );

        state.update_king_attacks();

        assert!(state.is_in_check);

        let mut squares: Vec<String> = state.check_rays.iter().map(|m| m.get_notation()).collect();

        squares.sort();
        let mut expected = ["d6", "e7", "f8", "b6", "d4", "e3"];
        expected.sort();

        assert_eq!(squares, expected);

        let blocker = Piece::new(PType::PAWN, Color::WHITE);
        let blocker_sq = Square::new(Rank::Six, File::D);
        state.board.set(blocker, &blocker_sq);

        state.update_king_attacks();

        assert!(state.is_in_check);

        let mut squares: Vec<String> = state.check_rays.iter().map(|m| m.get_notation()).collect();

        squares.sort();
        let mut expected = ["d4", "e3", "b6"];
        expected.sort();

        assert_eq!(squares, expected);
    }

    #[test]
    fn load_fen() {
        let mut state = GameState::new();
        state.load_fen("8/2b5/1N6/2K3qk/8/8/8/8 w - - 0 1");

        assert_eq!(
            state
                .board
                .get(&Square::new(Rank::Five, File::C))
                .expect("piece must exist here")
                .p_type,
            PType::KING
        );

        assert_eq!(
            state
                .board
                .get(&Square::new(Rank::Five, File::G))
                .expect("piece must exist here")
                .p_type,
            PType::QUEEN
        );
    }

    #[test]
    fn get_fen() {
        let mut state = GameState::new();
        let fens = [
            "8/2b5/1N6/2K3qk/8/8/8/8 w - - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rn1qkb1r/p1b1pppp/4n3/1p1p2B1/Q1N5/3B1N2/PPPPPPPP/R3K2R w KQkq - 0 1",
            "k7/8/8/8/8/8/8/7K w - - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "1nbqkbn1/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/1NBQKBN1 b - - 1 2",
        ];

        for fen in fens {
            state.load_fen(fen);

            assert_eq!(fen, state.get_fen());

            state.reset();
        }

        let mut state = GameState::new();
        state.load_fen("4k3/8/8/8/5p2/8/4P3/4K3 w - - 0 1");
        assert!(state.play_move(Move::from_str("e2", "e4", None)).is_ok());
        assert_eq!(state.get_fen(), "4k3/8/8/8/4Pp2/8/8/4K3 b - e3 0 1");

        // TODO: handle this case
        // let mut state = GameState::new();
        // state.load_fen("5k2/8/8/8/5p2/8/4P3/4KR2 w - - 0 1");
        // assert!(state.play_move(Move::from_str("e2", "e4", None)).is_ok());
        // assert_eq!(state.get_fen(), "5k2/8/8/8/4Pp2/8/8/4KR2 b - - 0 1");
    }

    struct TestCase {
        fen: String,
        moves: Vec<Move>,
        expected: String,
    }

    impl TestCase {
        fn new(fen: &str, moves: Vec<Move>, expected: &str) -> Self {
            Self {
                fen: fen.to_string(),
                moves,
                expected: expected.to_string(),
            }
        }
    }

    #[test]
    fn undo() {
        let tests = vec![
            TestCase::new(
                "r3k2r/8/8/8/6pP/8/8/R3K2R w KQkq - 0 1",
                vec![Move::from_str("e1", "g1", None)],
                "r3k2r/8/8/8/6pP/8/8/R3K2R w KQkq - 0 1",
            ),
            TestCase::new(
                "r3k2r/8/8/8/6pP/8/8/R3K2R w KQkq - 0 1",
                vec![Move::from_str("e1", "c1", None)],
                "r3k2r/8/8/8/6pP/8/8/R3K2R w KQkq - 0 1",
            ),
            TestCase::new(
                "r3k2r/8/8/8/6pP/8/8/R3K2R b KQkq - 0 1",
                vec![Move::from_str("e8", "g8", None)],
                "r3k2r/8/8/8/6pP/8/8/R3K2R b KQkq - 0 1",
            ),
            TestCase::new(
                "r3k2r/8/2q5/1P6/6p1/8/7P/R3K2R w KQkq - 0 1",
                vec![Move::from_str("h2", "h4", None)],
                "r3k2r/8/2q5/1P6/6p1/8/7P/R3K2R w KQkq - 0 1",
            ),
            TestCase::new(
                "r3k2r/8/2q5/1P6/6p1/8/7P/R3K2R w KQkq - 0 1",
                vec![Move::from_str("b5", "c6", None)],
                "r3k2r/8/2q5/1P6/6p1/8/7P/R3K2R w KQkq - 0 1",
            ),
            // en passant
            TestCase::new(
                "r3k2r/8/2q5/1P6/6pP/8/8/R3K2R b KQkq h3 0 1",
                vec![Move::from_str("g4", "h3", None)],
                "r3k2r/8/2q5/1P6/6pP/8/8/R3K2R b KQkq h3 0 1",
            ),
            // promotion
            TestCase::new(
                "r3k2r/5P2/2q5/1P6/6p1/7P/8/R3K2R w KQkq - 0 1",
                vec![Move::from_str(
                    "f7",
                    "f8",
                    Some(Piece::new(PType::QUEEN, Color::WHITE)),
                )],
                "r3k2r/5P2/2q5/1P6/6p1/7P/8/R3K2R w KQkq - 0 1",
            ),
        ];

        let mut state = GameState::new();

        for test in tests {
            state.reset();
            state.load_fen(&test.fen);

            for m in test.moves {
                assert!(state.play_move(m).is_ok());
                state.undo();
            }

            assert_eq!(state.get_fen(), test.expected);
            assert!(state.captures.is_empty());
        }
    }

    #[test]
    fn play_move() {
        let tests = vec![
            TestCase::new(
                "rnb1kbnr/pppppppp/1q6/4Q2B/8/2N4B/P1PPP2P/R3K2R w KQkq - 0 1",
                vec![
                    Move::from_str("e1", "f1", None),
                    Move::from_str("e8", "d8", None),
                ],
                "rnbk1bnr/pppppppp/1q6/4Q2B/8/2N4B/P1PPP2P/R4K1R w - - 2 2",
            ),
            TestCase::new(
                "r1bk1b1r/1ppppp1p/1q1n3n/p3Q1pB/8/2NP3B/P1PKP2P/1R4R1 w - - 0 1",
                vec![
                    Move::from_str("e5", "e7", None),
                    Move::from_str("d8", "e7", None),
                ],
                "r1b2b1r/1pppkp1p/1q1n3n/p5pB/8/2NP3B/P1PKP2P/1R4R1 w - - 0 2",
            ),
        ];

        let mut state = GameState::new();

        for test in tests {
            state.reset();
            state.load_fen(&test.fen);

            for m in test.moves {
                assert!(state.play_move(m).is_ok());
            }

            assert_eq!(state.get_fen(), test.expected);
        }
    }

    #[test]
    fn enpassant() {
        let tests = vec![
            TestCase::new(
                "1k6/8/8/8/5p2/8/4P3/4KR2 w - - 0 1",
                vec![Move::from_str("e2", "e4", None)],
                "1k6/8/8/8/4Pp2/8/8/4KR2 b - e3 0 1",
            ),
            TestCase::new(
                "1k6/8/2p5/8/8/8/4P3/4KR2 w - - 0 1",
                vec![Move::from_str("e2", "e4", None)],
                "1k6/8/2p5/8/4P3/8/8/4KR2 b - - 0 1",
            ),
            TestCase::new(
                "1k6/8/8/8/6p1/8/7P/4KR2 w - - 0 1",
                vec![Move::from_str("h2", "h4", None)],
                "1k6/8/8/8/6pP/8/8/4KR2 b - h3 0 1",
            ),
        ];

        let mut state = GameState::new();

        for test in tests {
            state.reset();
            state.load_fen(&test.fen);

            for m in test.moves {
                assert!(state.play_move(m).is_ok());
            }

            assert_eq!(state.get_fen(), test.expected);
        }
    }

    #[test]
    fn checkmate() {
        // test cases by https://github.com/jhlywa/chess.js
        let fens = vec![
            "8/5r2/4K1q1/4p3/3k4/8/8/8 w - - 0 7",
            "4r2r/p6p/1pnN2p1/kQp5/3pPq2/3P4/PPP3PP/R5K1 b - - 0 2",
            "r3k2r/ppp2p1p/2n1p1p1/8/2B2P1q/2NPb1n1/PP4PP/R2Q3K w kq - 0 8",
            "8/6R1/pp1r3p/6p1/P3R1Pk/1P4P1/7K/8 b - - 0 4",
        ];

        for fen in fens {
            let mut state = GameState::new();
            state.load_fen(fen);

            assert!(state.is_checkmate());
        }
    }

    #[test]
    fn insufficient_material() {
        // test cases by https://github.com/jhlywa/chess.js
        let fens = vec![
            "8/8/8/8/8/8/8/k6K w - - 0 1",
            "8/2N5/8/8/8/8/8/k6K w - - 0 1",
            "8/2b5/8/8/8/8/8/k6K w - - 0 1",
            "8/b7/3B4/8/8/8/8/k6K w - - 0 1",
            "8/b1B1b1B1/1b1B1b1B/8/8/8/8/1k5K w - - 0 1",
        ];

        for fen in fens {
            let mut state = GameState::new();
            state.load_fen(fen);

            assert!(state.is_insufficient_material());
        }

        let fens = vec![
            "8/2p5/8/8/8/8/8/k6K w - - 0 1",
            "5k1K/7B/8/6b1/8/8/8/8 b - - 0 1",
            "7K/5k1N/8/6b1/8/8/8/8 b - - 0 1",
            "7K/5k1N/8/4n3/8/8/8/8 b - - 0 1",
        ];

        for fen in fens {
            let mut state = GameState::new();
            state.load_fen(fen);

            assert!(!state.is_insufficient_material());
        }
    }

    #[test]
    fn stalemate() {
        // test cases by https://github.com/jhlywa/chess.js
        let fens = vec![
            "1R6/8/8/8/8/8/7R/k6K b - - 0 1",
            "8/8/5k2/p4p1p/P4K1P/1r6/8/8 w - - 0 2",
        ];

        for fen in fens {
            let mut state = GameState::new();
            state.load_fen(fen);

            assert!(state.is_stalemate());
        }

        let mut state = GameState::new();
        state.load_fen("R3k3/8/4K3/8/8/8/8/8 b - - 0 1");

        assert!(!state.is_stalemate());
    }

    #[test]
    fn threefold_repetition() {
        let mut state = GameState::new();
        state.load_fen("8/pp3p1k/2p2q1p/3r1P2/5R2/7P/P1P1QP2/7K b - - 2 30");
        let moves = vec![
            Move::from_str("f6", "e5", None),
            Move::from_str("e2", "h5", None),
            Move::from_str("e5", "f6", None),
            Move::from_str("h5", "e2", None),
            Move::from_str("d5", "e5", None),
            Move::from_str("e2", "d3", None),
            Move::from_str("e5", "d5", None),
            Move::from_str("d3", "e2", None),
        ];

        for m in moves {
            assert!(state.play_move(m).is_ok());
        }

        assert!(state.is_threefold_repetition());
    }

    // #[test]
    fn perft() {
        struct Perft {
            fen: &'static str,
            depth: u8,
            expected: usize,
        }

        let tests = [
            Perft {
                fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
                depth: 3,
                expected: 2812,
            },
            Perft {
                fen: "rnbqkbnr/p3pppp/2p5/1pPp4/3P4/8/PP2PPPP/RNBQKBNR w KQkq b6 0 4",
                depth: 3,
                expected: 23509,
            },
            Perft {
                fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 101",
                depth: 3,
                expected: 89890,
            },
            Perft {
                fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                depth: 3,
                expected: 62379,
            },
            Perft {
                fen: "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
                depth: 4,
                expected: 422333,
            },
            Perft {
                fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                depth: 3,
                expected: 97862,
            },
            Perft {
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                depth: 4,
                expected: 197281,
            },
        ];
        let mut state = GameState::new();

        for test in tests {
            state.reset();
            state.load_fen(&test.fen);
            assert_eq!(state.perft(test.depth, false), test.expected);
        }
    }
}
