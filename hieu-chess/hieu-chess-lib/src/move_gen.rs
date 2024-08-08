use crate::board::SIZE_0X88;
use crate::chess::{Color, GameState, Move};
use crate::constants::{
    BISHOP_DELTAS, BLACK_PAWN_DELTAS, KING_DELTAS, KNIGHT_DELTAS, QUEEN_DELTAS, ROOK_DELTAS,
    WHITE_PAWN_DELTAS,
};
use crate::piece::{PType, Piece, PROMOTION_PIECES};
use crate::square::Square;
use crate::utils;

pub struct MoveGen;

impl MoveGen {
    // TODO: movegen should not modify game state
    pub fn moves_for_square(state: &mut GameState, sq: Square) -> Vec<Move> {
        let board = &state.board;

        let Some(piece) = board.get(&sq) else {
            return vec![];
        };

        use PType::*;
        let pseudo_legal_moves: Vec<Move> = match piece.p_type {
            QUEEN | ROOK | BISHOP => Self::get_sliding_moves(state, sq, piece),
            KNIGHT => Self::get_knight_moves(state, sq),
            PAWN => Self::get_pawn_moves(state, sq, piece),
            KING => Self::get_king_moves(state, sq),
        };

        pseudo_legal_moves
            .into_iter()
            .filter(|m| {
                state.make_move(m.clone());

                let is_legal = if let Some(king) = state.get_current_king_sq() {
                    !state.is_attacked(king)
                } else {
                    true
                };

                state.undo();

                is_legal
            })
            .collect()
    }

    pub fn moves(state: &mut GameState, color: Color) -> Vec<Move> {
        let mut moves = vec![];

        for idx in 0..SIZE_0X88 {
            if utils::is_valid_idx(idx as u8).is_err() {
                continue;
            }

            let Some(piece) = state.board.get(&(idx as u8)) else {
                continue;
            };

            if piece.color == color {
                moves.append(&mut Self::moves_for_square(
                    state,
                    (idx as u8).try_into().unwrap(),
                ));
            }
        }

        moves
    }

    fn get_pawn_moves(state: &GameState, from_sq: Square, piece: &Piece) -> Vec<Move> {
        let board = &state.board;
        let mut moves: Vec<Move> = vec![];

        let deltas = match piece.color {
            Color::BLACK => BLACK_PAWN_DELTAS,
            Color::WHITE => WHITE_PAWN_DELTAS,
        };

        let mut can_go_forward = true;

        for delta in deltas {
            if let Ok(to_sq) = from_sq.add(*delta) {
                // regular forward move
                if delta % 2 == 0 {
                    if !can_go_forward {
                        continue;
                    }

                    // if there is any piece in front of the pawn, it can't move forward
                    let None = board.get(&to_sq) else {
                        can_go_forward = false;
                        continue;
                    };

                    // pawn can only move forward two squares if it hasn't already moved
                    if *delta == 32 || *delta == -32 {
                        if (from_sq.rank() == 1 && piece.color == Color::WHITE)
                            || (from_sq.rank() == 6 && piece.color == Color::BLACK)
                        {
                            moves.push(Move::new(from_sq, to_sq, None));
                        }
                        continue;
                    }

                    // a promotion move if the pawn reaches the last rank, otherwise it's a regular move
                    if to_sq.rank() == 0 || to_sq.rank() == 7 {
                        for mut p in PROMOTION_PIECES {
                            p.color = piece.color;

                            moves.push(Move::new(from_sq, to_sq, Some(p)));
                        }
                    } else {
                        moves.push(Move::new(from_sq, to_sq, None));
                    }
                } else {
                    // capture moves
                    let Some(p) = board.get(&to_sq) else {
                        // if the target square is empty and is also an en passant square, we can capture
                        if state.en_passant_sq == Some(to_sq) {
                            moves.push(Move::new(from_sq, to_sq, None));
                        }
                        continue;
                    };

                    if !utils::is_friendly(p, state.side_to_move) {
                        if to_sq.rank() == 0 || to_sq.rank() == 7 {
                            for mut p in PROMOTION_PIECES {
                                p.color = piece.color;

                                moves.push(Move::new(from_sq, to_sq, Some(p)));
                            }
                        } else {
                            moves.push(Move::new(from_sq, to_sq, None));
                        }
                    }
                }
            }
        }

        moves
    }

    fn get_king_moves(state: &GameState, from_sq: Square) -> Vec<Move> {
        let board = &state.board;

        let mut moves: Vec<Move> = vec![];
        let deltas = KING_DELTAS;

        for delta in deltas {
            if let Ok(to_sq) = from_sq.add(*delta) {
                if let Some(piece) = board.get(&to_sq) {
                    if utils::is_friendly(piece, state.side_to_move) {
                        continue;
                    }
                }
                let m = Move::new(from_sq, to_sq, None);

                // is there a better way to handle the castling logic?
                if state.is_castling(&m) {
                    let rights = state.get_castling_rights();
                    if (state.is_castling_kingside(&m) && !rights.0)
                        || (state.is_castling_queenside(&m) && !rights.1)
                        || state.is_occupied(&to_sq)
                        || state.is_in_check
                    {
                        continue;
                    }

                    let mut can_castle = true;
                    if state.is_castling_kingside(&m) {
                        for i in 1..3 {
                            let to = from_sq.add(i).expect("castling range must be valid");
                            if state.is_occupied(&to) || state.is_attacked(to) {
                                can_castle = false;
                                break;
                            }
                        }
                    } else if state.is_castling_queenside(&m) {
                        for i in 1..4 {
                            let to = from_sq.add(-1 * i).expect("castling range must be valid");
                            if state.is_occupied(&to) {
                                can_castle = false;
                                break;
                            }
                        }

                        for i in 1..3 {
                            let to = from_sq.add(-1 * i).expect("castling range must be valid");
                            if state.is_attacked(to) {
                                can_castle = false;
                                break;
                            }
                        }
                    } else {
                        can_castle = false;
                    }

                    if !can_castle {
                        continue;
                    }
                }

                moves.push(m);
            }
        }

        moves
    }

    fn get_knight_moves(state: &GameState, from_sq: Square) -> Vec<Move> {
        let board = &state.board;

        let mut moves: Vec<Move> = vec![];
        let deltas = KNIGHT_DELTAS;

        for delta in deltas {
            if let Ok(to_sq) = from_sq.add(*delta) {
                // knight can't capture friendly pieces, but can jump over them
                if let Some(piece) = board.get(&to_sq) {
                    if utils::is_friendly(piece, state.side_to_move) {
                        continue;
                    }
                }

                moves.push(Move::new(from_sq, to_sq, None));
            }
        }

        moves
    }

    fn get_sliding_moves(state: &GameState, from_sq: Square, piece: &Piece) -> Vec<Move> {
        let board = &state.board;

        let mut moves: Vec<Move> = vec![];
        let mut temp;

        use PType::*;
        let deltas: &[i8] = match piece.p_type {
            QUEEN => QUEEN_DELTAS,
            ROOK => ROOK_DELTAS,
            BISHOP => BISHOP_DELTAS,
            _ => &[],
        };

        for delta in deltas {
            temp = from_sq;

            while let Ok(to_sq) = temp.add(*delta) {
                // check if a piece is blocking the path
                if let Some(piece) = board.get(&to_sq) {
                    // if its an enemy piece, we can capture it, but can't go any further
                    if !utils::is_friendly(piece, state.side_to_move) {
                        moves.push(Move::new(from_sq, to_sq, None));
                    }

                    break;
                }

                moves.push(Move::new(from_sq, to_sq, None));
                temp = to_sq;
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        piece::{PType::*, Piece},
        square::{File, Rank},
    };

    use super::*;

    #[test]
    fn get_queen_moves_without_blockers() {
        let mut state = GameState::new();

        let sq = Square::new(Rank::Five, File::C);
        let p = Piece::new(QUEEN, Color::WHITE);

        state.board.set(p, &sq);

        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected = [
            "b6", "a7", "c6", "c7", "c8", "d6", "e7", "f8", "d5", "e5", "f5", "g5", "h5", "d4",
            "e3", "f2", "g1", "c4", "c3", "c2", "c1", "b4", "a3", "b5", "a5",
        ];

        expected.sort();
        moves.sort();

        assert!(moves.len() == expected.len());
        assert_eq!(expected.to_vec(), moves);
    }

    #[test]
    fn get_queen_moves_with_blockers() {
        let mut state = GameState::new();
        let sq = Square::new(Rank::Five, File::C);

        let p = Piece::new(QUEEN, Color::WHITE);
        let friendly_blocker = Piece::new(PAWN, Color::WHITE);
        let enemy_blocker = Piece::new(KNIGHT, Color::BLACK);

        state.board.set(p, &sq);
        state
            .board
            .set(friendly_blocker, &Square::new(Rank::Five, File::D));
        state
            .board
            .set(enemy_blocker, &Square::new(Rank::Four, File::C));

        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected = [
            "b6", "a7", "c6", "c7", "c8", "d6", "e7", "f8", "d4", "e3", "f2", "g1", "c4", "b4",
            "a3", "b5", "a5",
        ];

        expected.sort();
        moves.sort();

        assert!(moves.len() == expected.len());
        assert_eq!(expected.to_vec(), moves);
    }

    #[test]
    fn forward_white_pawn_moves() {
        let mut state = GameState::new();

        let pawn = Piece::new(PAWN, Color::WHITE);
        let blocking_piece = Piece::new(ROOK, Color::WHITE);
        let sq = Square::new(Rank::Two, File::E);

        state.board.set(pawn, &sq);

        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected = ["e3", "e4"];

        expected.sort();
        moves.sort();

        assert_eq!(expected.to_vec(), moves);
        state.en_passant_sq = None;

        state
            .board
            .set(blocking_piece, &Square::new(Rank::Four, File::E));
        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected = ["e3"];

        expected.sort();
        moves.sort();

        assert_eq!(expected.to_vec(), moves);
    }

    #[test]
    fn forward_black_pawn_moves() {
        let mut state = GameState::new();

        let pawn = Piece::new(PAWN, Color::BLACK);
        let blocking_piece = Piece::new(ROOK, Color::WHITE);
        let sq = Square::new(Rank::Seven, File::D);

        state.board.set(pawn, &sq);

        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected = ["d6", "d5"];

        expected.sort();
        moves.sort();

        assert_eq!(expected.to_vec(), moves);

        state
            .board
            .set(blocking_piece, &Square::new(Rank::Six, File::D));
        let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected: Vec<&str> = vec![];

        expected.sort();
        moves.sort();

        assert_eq!(expected.to_vec(), moves);
    }

    #[test]
    fn pawn_captures() {
        let mut state = GameState::new();
        let white_pawn = Piece::new(PAWN, Color::WHITE);
        let black_pawn = Piece::new(PAWN, Color::BLACK);
        let white_sq = Square::new(Rank::Three, File::E);
        let black_sq = Square::new(Rank::Four, File::F);

        state.board.set(white_pawn, &white_sq);
        state.board.set(black_pawn, &black_sq);

        let mut moves_for_white: Vec<String> = MoveGen::moves_for_square(&mut state, white_sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();
        state.side_to_move = Color::BLACK;
        let mut moves_for_black: Vec<String> = MoveGen::moves_for_square(&mut state, black_sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected_moves_for_white = ["e4", "f4"];
        let mut expected_moves_for_black = ["e3", "f3"];

        moves_for_white.sort();
        expected_moves_for_white.sort();
        moves_for_black.sort();
        expected_moves_for_black.sort();

        assert_eq!(moves_for_white, expected_moves_for_white);
        assert_eq!(moves_for_black, expected_moves_for_black);
    }

    #[test]
    fn pawn_promotion() {
        let mut state = GameState::new();
        let white_pawn = Piece::new(PAWN, Color::WHITE);
        let black_pawn = Piece::new(PAWN, Color::BLACK);
        let white_sq = Square::new(Rank::Seven, File::E);
        let black_sq = Square::new(Rank::Two, File::F);

        state.board.set(white_pawn, &white_sq);
        state.board.set(black_pawn, &black_sq);

        let mut moves_for_white: Vec<String> = MoveGen::moves_for_square(&mut state, white_sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();
        state.side_to_move = Color::BLACK;
        let mut moves_for_black: Vec<String> = MoveGen::moves_for_square(&mut state, black_sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        let mut expected_moves_for_white = ["e8"; 4];
        let mut expected_moves_for_black = ["f1"; 4];

        assert_eq!(moves_for_white, expected_moves_for_white);
        assert_eq!(moves_for_black, expected_moves_for_black);
    }

    #[test]
    fn moves_in_check() {
        let mut state = GameState::new();
        state.load_fen("8/2bN4/8/2K3qk/8/8/3Q4/8 w - - 0 1");
        // state.load_fen("1bn5/3N4/p7/2K3qk/4P3/8/3Q4/8 w - - 0 1");
        let sq = Square::new(Rank::Two, File::D);

        let moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
            .iter()
            .map(|m| m.to.get_notation())
            .collect();

        assert_eq!(moves, ["d5", "g5"]);
        assert!(state.captures.is_empty());

        // let sq = Square::new(Rank::Four, File::E);
        // let moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
        //     .iter()
        //     .map(|m| m.to.get_notation())
        //     .collect();
        // println!("{:?}", moves);
    }

    struct TestCase {
        fen: String,
        expected: Vec<&'static str>,
    }

    impl TestCase {
        fn new(fen: &str, mut expected: Vec<&'static str>) -> Self {
            expected.sort();

            Self {
                fen: fen.to_string(),
                expected,
            }
        }

        fn match_case(&self, result: &mut Vec<String>) -> bool {
            result.sort();

            let matching = self
                .expected
                .iter()
                .zip(result.iter())
                .filter(|&(a, b)| a == b)
                .count();

            matching == result.len() && matching == self.expected.len()
        }
    }

    #[test]
    fn king_moves() {
        let mut state = GameState::new();

        let tests = [
            TestCase::new(
                "rnb1kbnr/pppppppp/1q6/4Q2B/8/2N4B/P1PPP2P/R3K2R b KQkq - 0 1",
                vec!["d8"],
            ),
            TestCase::new("1r6/2k4q/8/n7/2p5/5B2/3R1K2/8 b - - 0 1", vec!["b6", "c8"]),
            TestCase::new(
                "1r6/2k4q/8/n7/2p5/5B2/3R1K2/8 w - - 0 1",
                vec!["e3", "e2", "e1", "f1", "g1", "g2", "g3"],
            ),
            TestCase::new(
                "1r4r1/2k4q/8/n7/2p5/5B2/3R1K2/8 w - - 0 1",
                vec!["e3", "e2", "e1", "f1"],
            ),
            TestCase::new(
                "rnb1kbnr/pppppppp/5q2/8/3P4/8/PPPPP1PP/R3K2R w KQkq - 0 1",
                vec!["d1", "c1"],
            ),
            TestCase::new(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
                vec!["d1", "c1", "f1", "g1"],
            ),
            TestCase::new(
                "rnb1kbnr/pppppppp/1q6/4Q2B/8/2N4B/P1PPP2P/R3K2R w KQkq - 0 1",
                vec!["d1", "f1", "c1"],
            ),
            TestCase::new(
                "r3k2r/8/7q/n7/2p5/5B2/3R1K2/8 b kq - 1 1",
                vec!["g8", "f8", "f7", "e7"],
            ),
            TestCase::new(
                "r3k2r/p1ppqpb1/bn1Ppnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
                vec!["f8", "g8", "d8", "c8"],
            ),
        ];

        for test in tests {
            state.reset();
            state.load_fen(&test.fen);

            let sq = if state.side_to_move == Color::WHITE {
                state.white_king_square.unwrap()
            } else {
                state.black_king_square.unwrap()
            };

            let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
                .iter()
                .map(|m| m.to.get_notation())
                .collect();

            assert!(
                test.match_case(&mut moves),
                "fen: {} moves: {:?}",
                test.fen,
                moves
            );
        }
    }

    #[test]
    fn enpassant_moves() {
        let mut state = GameState::new();

        let tests = [
            TestCase::new(
                "r3k2r/8/4p2q/n1pP4/8/5B2/3R1K2/8 w kq c6 1 1",
                vec!["d6", "e6", "c6"],
            ),
            TestCase::new(
                "r3k2r/3n4/7q/1Pp5/8/5B2/3R1K2/8 w kq c6 1 1",
                vec!["b6", "c6"],
            ),
            TestCase::new(
                "r3k2r/3n4/7q/8/1Pp5/5B2/3R1K2/8 b kq b3 1 1",
                vec!["b3", "c3"],
            ),
        ];
        let squares: Vec<&str> = vec!["d5", "b5", "c4"];

        for (i, test) in tests.iter().enumerate() {
            state.reset();
            state.load_fen(&test.fen);

            let sq: Square = squares[i].try_into().unwrap();

            let mut moves: Vec<String> = MoveGen::moves_for_square(&mut state, sq)
                .iter()
                .map(|m| m.to.get_notation())
                .collect();

            assert!(
                test.match_case(&mut moves),
                "fen: {} moves: {:?}",
                test.fen,
                moves
            );
        }
    }

    #[test]
    fn moves() {
        let mut state = GameState::new();
        // state.load_fen("8/8/3p4/KPp4r/R4p1k/8/4P1P1/8 w - c6 0 1");
        state.load_fen("8/8/3p4/KPp4r/R4p1k/8/4P1P1/8 w - c6 0 1");
        let a: Vec<(String, String)> =
            MoveGen::moves_for_square(&mut state, "b5".try_into().unwrap())
                .iter()
                .map(|m| (m.from.get_notation(), m.to.get_notation()))
                .collect();
        println!("{:?}", a);
    }
}
