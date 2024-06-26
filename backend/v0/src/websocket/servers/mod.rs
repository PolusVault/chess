use self::in_memory::{Player, PlayerStatus};

use super::session::{Message, Session};
use crate::types::Color;
use std::fmt::Debug;

pub mod in_memory;
pub mod redis;

pub trait WsServer: Unpin + 'static + Debug {
    type Session;
    type Game;

    fn get_session(&self, id: &str) -> Option<&Self::Session>;
    fn create_session(&mut self, id: &str, session: Session);
    fn delete_session(&mut self, id: &str);
    fn update_session_name(&mut self, id: &str, name: &str);

    fn get_game(&self, id: &str) -> Option<&Self::Game>;

    /// Create a game and join player one to the game
    ///
    /// Returns the ID of the game if created successfully, None if not.
    fn create_game(&mut self, name: &str, player_one: &str, color: Color) -> Option<String>;
    fn join_game(&mut self, game_id: &str, player_id: &str);
    fn leave_game(&mut self, game_id: &str, player_id: &str);
    fn delete_game(&mut self, id: &str);
    fn update_game_state(&mut self, player_id: &str, player_status: PlayerStatus);

    fn make_move(&mut self, chess_move: &str, player_id: &str);

    fn get_player_one(&self, game_id: &str) -> Option<Player>;
    fn get_player_two(&self, game_id: &str) -> Option<Player>;

    /// Send a message to a session actor
    fn send(&self, id: &str, msg: Message);
}
