use super::WsServer;
use crate::types::Color;
use crate::websocket::session::{Message, Session};
use serde::*;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum DrawCondition {
    InsufficientMaterial,
    Stalemate,
    Repetition,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WinLoseCondition {
    Checkmate,
    Resign,
    Overtime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    win: Option<WinLoseCondition>,
    lose: Option<WinLoseCondition>,
}

#[derive(Debug)]
pub struct GameState {
    pub draw: Option<DrawCondition>,
    pub player_one: PlayerStatus,
    pub player_two: PlayerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: Color,
}

impl From<&Session> for Player {
    fn from(value: &Session) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            color: value.color,
        }
    }
}

#[derive(Debug)]
pub struct Game {
    pub name: String,
    // The player who created the game will always be player_one
    pub player_one_id: String,
    pub player_two_id: Option<String>,
    pub game_state: GameState,
}

impl Game {
    pub fn new(name: &str, player_one_id: String) -> Self {
        Self {
            name: name.to_owned(),
            player_one_id,
            player_two_id: None,
            game_state: GameState {
                draw: None,
                player_one: PlayerStatus {
                    win: None,
                    lose: None,
                },
                player_two: PlayerStatus {
                    win: None,
                    lose: None,
                },
            },
        }
    }
}

#[derive(Default, Debug)]
pub struct InMemoryServer {
    games: HashMap<String, Game>,
    sessions: HashMap<String, Session>,
}

impl WsServer for InMemoryServer {
    type Session = Session;
    type Game = Game;

    fn get_game(&self, id: &str) -> Option<&Self::Game> {
        self.games.get(id)
    }

    fn create_game(&mut self, name: &str, player_one_id: &str, color: Color) -> Option<String> {
        if player_one_id.is_empty() {
            return None;
        }

        let id = nanoid::nanoid!(10);
        self.games
            .insert(id.clone(), Game::new(name, player_one_id.to_owned()));

        if let Some(session) = self.sessions.get_mut(player_one_id) {
            session.color = color;
            session.joined_game = Some(id.to_owned());
        }

        Some(id)
    }

    fn join_game(&mut self, game_id: &str, player_id: &str) {
        let mut player_two_color = Color::None;

        if let Some(game) = self.games.get(game_id) {
            if let Some(player_one) = self.sessions.get(&game.player_one_id) {
                player_two_color = match player_one.color {
                    Color::Black => Color::White,
                    Color::White => Color::Black,
                    _ => panic!("invalid color"),
                }
            }
        }

        if let Some(session) = self.sessions.get_mut(player_id) {
            if let Some(game) = self.games.get_mut(game_id) {
                if game.player_two_id.is_some() {
                    println!("game is full, can't join");
                } else {
                    session.color = player_two_color;
                    session.joined_game = Some(game_id.to_owned());
                    game.player_two_id = Some(player_id.to_owned());
                }
            }
        }
    }

    fn leave_game(&mut self, game_id: &str, player_id: &str) {
        if let Some(game) = self.games.get_mut(game_id) {
            // if player_one leaves, we delete the game
            if game.player_one_id == player_id {
                self.delete_game(game_id);
                return;
            }

            if let Some(player_two) = &game.player_two_id {
                if player_two.as_str() == player_id {
                    game.player_two_id = None;
                }
            }
        }
    }

    fn delete_game(&mut self, id: &str) {
        self.games.remove(id);
    }

    fn update_game_state(&mut self, player_id: &str, player_status: PlayerStatus) {
        if let Some(session) = self.sessions.get_mut(player_id) {
            if let Some(game_id) = &session.joined_game {
                if let Some(game) = self.games.get_mut(game_id) {
                    if game.player_one_id == player_id {
                        game.game_state.player_one = player_status;
                    } else if let Some(player_two_id) = &game.player_two_id {
                        if player_two_id == player_id {
                            game.game_state.player_two = player_status;
                        }
                    }

                    let game_id = game_id.clone();

                    // game.game_state.is_finished = true;

                    // This is temporary
                    // Delete the game since it is finished
                    session.joined_game = None;
                    session.color = Color::None;

                    if let Some(player_two_id) = &game.player_two_id {
                        if let Some(player_two_session) = self.sessions.get_mut(player_two_id) {
                            player_two_session.joined_game = None;
                            player_two_session.color = Color::None;
                        }
                    }
                    self.games.remove(&game_id);
                }
            }
        }
    }

    fn get_session(&self, id: &str) -> Option<&Self::Session> {
        self.sessions.get(id)
    }

    fn create_session(&mut self, id: &str, session: Session) {
        self.sessions.insert(id.to_owned(), session);
    }

    fn delete_session(&mut self, id: &str) {
        if let Some(session) = self.sessions.remove(id) {
            if let Some(game_id) = session.joined_game {
                self.leave_game(&game_id, id)
            }
        }
    }

    fn update_session_name(&mut self, id: &str, name: &str) {
        if let Some(session) = self.sessions.get_mut(id) {
            session.name = name.to_owned()
        }
    }

    fn make_move(&mut self, chess_move: &str, player_id: &str) {
        if let Some(player) = self.get_session(player_id) {
            if let Some(game_id) = &player.joined_game {
                if let Some(game) = self.get_game(&game_id) {
                    if game.player_one_id == player_id {
                        if let Some(player_two_id) = &game.player_two_id {
                            self.send(player_two_id, Message(chess_move.to_owned()));
                        }
                    } else {
                        self.send(&game.player_one_id, Message(chess_move.to_owned()));
                    }
                }
            }
        }
    }

    fn get_player_one(&self, game_id: &str) -> Option<Player> {
        if let Some(game) = self.get_game(game_id) {
            if let Some(session) = self.get_session(&game.player_one_id) {
                return Some(session.into());
                // return Some(Player {
                //     id: session.id.clone(),
                //     name: session.name.clone(),
                //     color: session.color,
                // });
            }
        }

        None
    }

    fn get_player_two(&self, game_id: &str) -> Option<Player> {
        if let Some(game) = self.get_game(game_id) {
            if let Some(player_two) = &game.player_two_id {
                if let Some(session) = self.get_session(player_two) {
                    return Some(session.into());
                }
            }
        }

        None
    }

    fn send(&self, id: &str, msg: Message) {
        if let Some(session) = self.get_session(id) {
            (&session.addr).do_send(msg);
        }
    }
}
