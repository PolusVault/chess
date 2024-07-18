use actix::prelude::*;
use serde::*;

use super::{servers::in_memory::PlayerStatus, session::Message};
use crate::types::Color;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Error,
    Connect,
    Disconnect,
    CreateGame,
    JoinGame,
    UpdateName,
    OpponentJoined,
    MakeMove,
    UpdateGameState,
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct Connect {
    // #[serde(rename(serialize = "type"))]
    #[serde(skip_serializing)]
    pub m_type: Type,

    pub id: String,

    #[serde(skip_serializing)]
    /// the address of the session actor
    pub addr: Recipient<Message>,
}

impl Connect {
    pub fn new(id: String, addr: Recipient<Message>) -> Self {
        Self {
            m_type: Type::Connect,
            id,
            addr,
        }
    }
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct Disconnect {
    // #[serde(rename(serialize = "type"))]
    #[serde(skip_serializing)]
    pub m_type: Type,

    pub id: String,
}

impl Disconnect {
    pub fn new(id: String) -> Self {
        Self {
            m_type: Type::Disconnect,
            id,
        }
    }
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct CreateGame {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub player_id: String,
    #[serde(skip_serializing)]
    pub name: String,
    #[serde(skip_serializing)]
    pub color: Color,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct JoinGame {
    pub game_id: String,
    #[serde(skip_deserializing)]
    pub player_id: String,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
// #[serde(rename_all = "lowercase")]
pub struct MakeMove {
    pub from: String,
    pub to: String,
    pub promotion_piece: Option<String>,
    #[serde(skip_deserializing)]
    pub game_id: String,
    #[serde(skip_deserializing)]
    pub player_id: String,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct UpdateName {
    pub name: String,
    #[serde(skip_deserializing)]
    pub player_id: String,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct UpdateGameState {
    pub new_status: PlayerStatus,
    #[serde(skip_deserializing)]
    pub player_id: String,
}

/// Represents a message that will be sent to the client
/// It is not meant to be sent between actors
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub m_type: Type,
    pub payload: serde_json::Value,
}
