use serde::{Deserialize, Serialize};

use crate::game::*;

pub mod id;

pub use self::id::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg")]
pub enum MachMessage {
	Handshake(Handshake),
	HandshakeOk(HandshakeOk),
	HandshakeFailure(HandshakeFailure),
	CreateGameRequest(CreateGameRequest),
	CreateGameResponse(CreateGameResponse),
	GetInviteTokenRequest(GetInviteTokenRequest),
	GetInviteTokenResponse(GetInviteTokenResponse),
	JoinGameRequest(JoinGameRequest),
	JoinGameResponse(JoinGameResponse),
	GetGameStateRequest(GetGameStateRequest),
	GetGameStateResponse(GetGameStateResponse),
	GameMoveRequest(GameMoveRequest),
	GameMoveResponse(GameMoveResponse),
	GameMoveHappened(GameMoveHappened),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
	pub versions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeOk {
	pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeFailure {
	pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameRequest {
	pub id: Id,
	pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGameResponse {
	pub id: Id,
	pub game_id: ServerId,
	pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInviteTokenRequest {
	pub id: Id,
	pub game_id: ServerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetInviteTokenResponse {
	pub id: Id,
	pub invite_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGameRequest {
	pub id: Id,
	pub game_id: Id,
	pub invite_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGameResponse {
	pub id: Id,
	pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGameStateRequest {
	pub id: Id,
	pub game_id: ServerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGameStateResponse {
	pub id: Id,
	pub game_state: GameState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMoveRequest {
	pub id: Id,
	pub game_id: ServerId,
	pub move_start: BoardIndex,
	pub move_end: BoardIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMoveResponse {
	pub id: Id,
	pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMoveHappened {
	pub game_id: ServerId,
	pub move_start: BoardIndex,
	pub move_end: BoardIndex,
}
