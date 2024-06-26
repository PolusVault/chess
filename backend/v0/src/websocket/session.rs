use actix::prelude::*;
use actix::{Actor, Addr, Handler, Recipient, StreamHandler};
use actix_web_actors::ws::{self, CloseReason};
use nanoid::nanoid;

use super::messages::{MakeMove, UpdateGameState};
use super::{
    messages::{ClientMessage, Connect, CreateGame, Disconnect, JoinGame, Type, UpdateName},
    server::WsChessServer,
    servers::WsServer,
};
use crate::types::Color;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Debug)]
pub struct Session {
    pub id: String,
    /// The address of the session actor
    pub addr: Recipient<Message>,
    /// The name of the player
    pub name: String,
    pub joined_game: Option<String>,
    pub color: Color,
}

pub struct SessionActor<T: WsServer> {
    pub id: String,
    pub server_addr: Addr<WsChessServer<T>>,
}

impl<T: WsServer> SessionActor<T> {
    pub fn new(server_addr: Addr<WsChessServer<T>>) -> Self {
        Self {
            id: nanoid!(10),
            server_addr,
        }
    }
}

impl<T: WsServer> Actor for SessionActor<T> {
    type Context = ws::WebsocketContext<Self>;

    // Whenever the actor is started, we send a Connect message with
    // the current actor's address to the WsChessServer actor to register a new session
    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();

        self.server_addr
            .send(Connect::new(self.id.clone(), addr.recipient()))
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server_addr.do_send(Disconnect::new(self.id.clone()));
        Running::Stop
    }
}

impl<T: WsServer> Handler<Message> for SessionActor<T> {
    type Result = ();

    /// Forward a message from the websocket server to the client
    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}

impl<T: WsServer> StreamHandler<Result<ws::Message, ws::ProtocolError>> for SessionActor<T> {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = item.expect("Unable to handle Websocket message");

        match msg {
            ws::Message::Text(text) => {
                match parse_text(text.into(), &self.id, &self.server_addr) {
                    Ok(_) => {}
                    Err(_) => {
                        ctx.close(Some(CloseReason {
                            code: ws::CloseCode::Normal,
                            description: Some(format!("Error")),
                        }));
                        ctx.stop();
                    }
                };
            }
            ws::Message::Close(_) => {
                // ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

fn parse_text<T: WsServer>(
    text: String,
    id: &str,
    server_addr: &Addr<WsChessServer<T>>,
) -> Result<(), serde_json::Error> {
    let msg: ClientMessage = serde_json::from_str(&text)?;

    match msg.m_type {
        Type::CreateGame => {
            let mut msg = serde_json::from_value::<CreateGame>(msg.payload)?;
            // attach the session id to the player_id field
            // so the client doesn't have to include their session id everytime they send a message
            msg.player_id = id.to_owned();
            server_addr.do_send(msg);
        }

        Type::JoinGame => {
            let mut msg = serde_json::from_value::<JoinGame>(msg.payload)?;
            msg.player_id = id.to_owned();
            server_addr.do_send(msg);
        }

        Type::UpdateName => {
            let mut msg = serde_json::from_value::<UpdateName>(msg.payload)?;
            msg.player_id = id.to_owned();
            server_addr.do_send(msg);
        }

        Type::MakeMove => {
            let mut msg = serde_json::from_value::<MakeMove>(msg.payload)?;
            msg.player_id = id.to_owned();
            server_addr.do_send(msg);
        }

        Type::UpdateGameState => {
            let mut msg = serde_json::from_value::<UpdateGameState>(msg.payload)?;
            msg.player_id = id.to_owned();
            server_addr.do_send(msg);
        }

        _ => {}
    }

    Ok(())
}
