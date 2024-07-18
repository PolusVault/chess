use actix::{Actor, Context, Handler};
use serde_json;

use super::{
    messages::{
        Connect, CreateGame, Disconnect, JoinGame, MakeMove, Type, UpdateGameState, UpdateName,
    },
    servers::WsServer,
};
use crate::{
    types::{ChessMove, Color},
    websocket::{
        messages::ClientMessage,
        session::{Message, Session},
    },
};
pub struct WsChessServer<T: WsServer> {
    inner_server: T,
    player_count: u8,
}

impl<T: WsServer> WsChessServer<T> {
    pub fn new(inner_server: T) -> Self {
        Self {
            inner_server,
            player_count: 0,
        }
    }
}

impl<T: WsServer> Actor for WsChessServer<T> {
    type Context = Context<Self>;
}

impl<T: WsServer> Handler<Connect> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        if self.player_count >= 3 {
            println!("player count limit reached, can't connect");
            return;
        }
        println!("Someone connected!");

        let id = msg.id.clone();

        self.inner_server.create_session(
            &msg.id,
            Session {
                id: id.clone(),
                color: Color::None,
                name: String::new(),
                addr: Clone::clone(&msg.addr),
                joined_game: None,
            },
        );

        let client_msg = serde_json::to_string(&ClientMessage {
            m_type: Type::Connect,
            payload: serde_json::to_value(msg).unwrap(),
        })
        .expect("unable to parse connect message");

        self.inner_server.send(id.as_str(), Message(client_msg));

        self.player_count += 1;
    }
}

impl<T: WsServer> Handler<Disconnect> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        println!("Someone disconnected!");

        self.inner_server.delete_session(&msg.id);

        println!("{:?}", self.inner_server);
        self.player_count = self.player_count.saturating_sub(1);
    }
}

impl<T: WsServer> Handler<CreateGame> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, mut msg: CreateGame, _: &mut Self::Context) -> Self::Result {
        if self.player_count >= 3 {
            println!("player limit reached, cant create games");
            return;
        }
        println!("Creating game");

        let player_id = msg.player_id.clone();

        println!("{}", player_id);

        let id = self
            .inner_server
            .create_game(&msg.name, &msg.player_id, msg.color)
            .expect("Player Id must be present when creating game");

        // attach the game id to the message to send back to the client
        msg.id = id;

        let client_msg = serde_json::to_string(&ClientMessage {
            m_type: Type::CreateGame,
            payload: serde_json::to_value(msg).unwrap(),
        })
        .expect("unable to parse CreateGame message");

        self.inner_server
            .send(player_id.as_str(), Message(client_msg));
    }
}

impl<T: WsServer> Handler<JoinGame> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: JoinGame, _: &mut Self::Context) -> Self::Result {
        if self.player_count >= 3 {
            return;
        }
        println!("Joining game");

        let player_id = msg.player_id.clone();

        self.inner_server.join_game(&msg.game_id, &msg.player_id);

        let player_one = self.inner_server.get_player_one(&msg.game_id);
        let player_one_id = match &player_one {
            Some(p) => Some(p.id.clone()),
            None => panic!("Joining an non-existent game"),
        };
        let player_two_msg = serde_json::to_string(&ClientMessage {
            m_type: Type::OpponentJoined,
            payload: serde_json::to_value(player_one).unwrap(),
        })
        .expect("unable to parse client message");

        let player_two = self.inner_server.get_player_two(&msg.game_id);
        let player_one_msg = serde_json::to_string(&ClientMessage {
            m_type: Type::OpponentJoined,
            payload: serde_json::to_value(player_two).unwrap(),
        })
        .expect("unable to parse client message");

        self.inner_server
            .send(player_id.as_str(), Message(player_two_msg));

        if let Some(player_one_id) = player_one_id {
            self.inner_server
                .send(&player_one_id.as_str(), Message(player_one_msg));
        }
    }
}

impl<T: WsServer> Handler<MakeMove> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: MakeMove, _: &mut Self::Context) -> Self::Result {
        let player_id = msg.player_id.clone();

        let client_msg = serde_json::to_string(&ClientMessage {
            m_type: Type::MakeMove,
            payload: serde_json::to_value::<ChessMove>(msg.into()).unwrap(),
        })
        .expect("failed to parse connect message");

        self.inner_server.make_move(&client_msg, &player_id);
    }
}

impl<T: WsServer> Handler<UpdateGameState> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: UpdateGameState, _: &mut Self::Context) -> Self::Result {
        // TODO: IMPLEMENT
        // we want to send the client a message to update their game state, not the other way around
    }
}

impl<T: WsServer> Handler<UpdateName> for WsChessServer<T> {
    type Result = ();

    fn handle(&mut self, msg: UpdateName, _: &mut Self::Context) -> Self::Result {
        self.inner_server
            .update_session_name(&msg.player_id, &msg.name);
    }
}
