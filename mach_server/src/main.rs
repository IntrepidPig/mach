use std::sync::Arc;

use futures::{sink::SinkExt, stream::StreamExt};
use tokio::{
	net::{TcpListener, TcpStream},
	sync::Mutex,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use mach::{game::*, proto::*};

fn setup_logging() -> Result<(), ()> {
	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!("[{}][{}] {}", record.target(), record.level(), message))
		})
		.level(log::LevelFilter::Warn)
		.level_for("mach_server", log::LevelFilter::Trace)
		.level_for("mach", log::LevelFilter::Trace)
		.chain(std::io::stderr())
		.apply()
		.map_err(|_| ())
}

#[tokio::main]
async fn main() {
	setup_logging().expect("Failed to setup logger");

	let addr = "127.0.0.1:8099";
	let mut listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");

	let global_state = Arc::new(Mutex::new(GlobalState::new()));

	let server = async move {
		let mut incoming = listener.incoming();
		while let Some(socket_res) = incoming.next().await {
			match socket_res {
				Ok(socket) => {
					println!("Got connection from {:?}", socket.peer_addr());
					tokio::spawn(init(socket, Arc::clone(&global_state)));
				}
				Err(e) => {
					println!("Failed to accept connection: {}", e);
				}
			}
		}
	};

	println!("Running mach backend server on {}", addr);

	server.await;
}

pub struct ConnectionState {
	ws_stream: WebSocketStream<TcpStream>,
	protocol_version: Option<u32>,
	global_state: Arc<Mutex<GlobalState>>,
	client_handle: ClientHandle,
}

impl ConnectionState {
	pub async fn run(mut self) {
		self.perform_handshake().await.unwrap();
		loop {
			match self.ws_stream.next().await {
				Some(Ok(Message::Text(data))) => {
					let message: MachMessage = json::from_str(&data).unwrap();
					self.handle_message(message).await.unwrap();
				}
				Some(Ok(Message::Close(_close_frame))) => {
					log::trace!("Websocket connection closed");
					break;
				}
				Some(Ok(u)) => {
					log::info!("Got other message while waiting for initialization: {:?}", u);
					self.ws_stream.close(None).await.unwrap();
					break;
				}
				Some(Err(e)) => {
					log::warn!(
						"Encountered an error with the websocket while waiting for initialization: {}",
						e
					);
					break;
				}
				None => {
					log::trace!("Websocket connection closed");
					break;
				}
			}
		}
	}

	async fn handle_message(&mut self, message: MachMessage) -> Result<(), ()> {
		log::trace!("Got message: {:?}", message);
		match message {
			MachMessage::CreateGameRequest(create) => {
				let mut game_state = GameState::new();
				game_state.board.set_standard();
				let mut global_lock = self.global_state.lock().await;
				let server_id = global_lock.next_server_id();
				let game = Game {
					client_handle: self.client_handle,
					client_color: create.color,
					other_client_handle: None,
					id: create.id,
					server_id: *server_id,
					game_state,
					invite_tokens: Vec::new(),
				};
				global_lock.games.push(game);
				drop(global_lock);
				let message = MachMessage::CreateGameResponse(CreateGameResponse {
					id: create.id,
					game_id: server_id,
					color: create.color,
				});
				self.ws_stream
					.send(Message::Text(json::to_string(&message).unwrap()))
					.await
					.unwrap();
			}
			MachMessage::GetInviteTokenRequest(get) => {
				let mut global_lock = self.global_state.lock().await;
				// TODO only do this once to aviod exhaustion
				let new_invite_token = global_lock.next_invite_token();
				for game in &mut global_lock.games {
					if (game.client_handle == self.client_handle && game.id == get.game_id)
						|| game.server_id == get.game_id
					{
						game.invite_tokens.push(new_invite_token.clone());
						self.ws_stream
							.send(Message::Text(
								json::to_string(&MachMessage::GetInviteTokenResponse(GetInviteTokenResponse {
									id: get.id,
									invite_token: new_invite_token.clone(),
								}))
								.unwrap(),
							))
							.await
							.unwrap();
					}
				}
				drop(global_lock);
			}
			MachMessage::JoinGameRequest(join) => {
				let mut global_lock = self.global_state.lock().await;
				for game in &mut global_lock.games {
					if game.invite_tokens.contains(&join.invite_token) {
						let success = if game.other_client_handle.is_some() {
							true
						} else {
							game.other_client_handle = Some(self.client_handle);
							false
						};
						let message = MachMessage::JoinGameResponse(JoinGameResponse { id: join.id, success });
						self.ws_stream
							.send(Message::Text(json::to_string(&message).unwrap()))
							.await
							.unwrap();
					}
				}
			}
			MachMessage::GetGameStateRequest(get) => {
				let global_lock = self.global_state.lock().await;
				for game in &global_lock.games {
					if game.server_id == get.game_id {
						let message = MachMessage::GetGameStateResponse(GetGameStateResponse {
							id: get.id,
							game_state: game.game_state.clone(),
						});
						self.ws_stream
							.send(Message::Text(json::to_string(&message).unwrap()))
							.await
							.unwrap();
					}
				}
			}
			MachMessage::GameMoveRequest(req) => {
				let mut global_lock = self.global_state.lock().await;
				for game in &mut global_lock.games {
					let game: &mut Game = game;
					if game.server_id == req.game_id {
						let start_piece =
							std::mem::replace(game.game_state.board.get_board_index_mut(req.move_start), None);
						let _end_piece =
							std::mem::replace(game.game_state.board.get_board_index_mut(req.move_end), start_piece);
						let message = MachMessage::GameMoveResponse(GameMoveResponse {
							id: req.id,
							success: true,
						});
						self.ws_stream
							.send(Message::Text(json::to_string(&message).unwrap()))
							.await
							.unwrap();
					}
				}
			}
			m => {
				log::debug!("Got unexpected message from client: {:?}", m);
			}
		}
		Ok(())
	}

	async fn perform_handshake(&mut self) -> Result<(), ()> {
		self.ws_stream
			.send(Message::Text(
				json::to_string(&MachMessage::Handshake(Handshake { versions: vec![0] })).unwrap(),
			))
			.await
			.unwrap();
		let version = match self.ws_stream.next().await.unwrap().unwrap() {
			Message::Text(text) => {
				let msg: MachMessage = json::from_str(&text).unwrap();
				match msg {
					MachMessage::HandshakeOk(ok) => {
						log::trace!("Got HandshakeOk from client, with version: {}", ok.version);
						ok.version
					}
					MachMessage::HandshakeFailure(failure) => {
						log::trace!("Got handshake failure from client, with reason: {}", failure.reason);
						return Err(());
					}
					m => {
						log::trace!(
							"Got unexpected message from client while waiting for Handshake Acknowledgement: {:?}",
							m
						);
						return Err(());
					}
				}
			}
			m => {
				log::trace!("Got unexpected message from client, {:?}", m);
				return Err(());
			}
		};
		self.protocol_version = Some(version);
		Ok(())
	}
}

pub async fn init(socket: TcpStream, global_state: Arc<Mutex<GlobalState>>) {
	let ws_stream = tokio_tungstenite::accept_async(socket).await.unwrap();
	let client_handle = global_state.lock().await.next_client_handle();
	let connection_state = ConnectionState {
		ws_stream,
		protocol_version: None,
		global_state,
		client_handle,
	};
	connection_state.run().await;
}

pub struct GlobalState {
	games: Vec<Game>,
	client_handle_tracker: ClientHandle,
	id_tracker: i32,
	invite_token_tracker: Vec<u8>,
}

impl GlobalState {
	pub fn new() -> Self {
		Self {
			games: Vec::new(),
			client_handle_tracker: 1,
			id_tracker: -1,
			invite_token_tracker: String::from("aaaaaaaa").into_bytes(),
		}
	}
}

impl GlobalState {
	pub fn next_client_handle(&mut self) -> ClientHandle {
		let current = self.client_handle_tracker;
		self.client_handle_tracker += 1;
		current
	}

	pub fn next_server_id(&mut self) -> ServerId {
		let current = self.id_tracker;
		self.id_tracker -= 1;
		ServerId::new(Id::new(current))
	}

	pub fn next_invite_token(&mut self) -> String {
		next_invite_token(&mut self.invite_token_tracker)
	}
}

fn next_invite_token(invite_token_tracker: &mut Vec<u8>) -> String {
	let current = invite_token_tracker.clone();
	let bytes = invite_token_tracker;
	let mut index = bytes.len() - 1;
	loop {
		if bytes[index] == b'z' {
			if index == 0 {
				panic!("Invite token string exhausted");
			} else {
				bytes[index] = b'a';
				index -= 1;
				continue;
			}
		} else {
			bytes[index] += 1;
			break;
		}
	}
	String::from_utf8(current).unwrap()
}

#[test]
fn invite_token_test() {
	let mut start = String::from("aaaaaaaa").into_bytes();
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaaaa"));
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaaab"));
	let mut start = String::from("aaaaaaaz").into_bytes();
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaaaz"));
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaaba"));
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaabb"));
	let mut start = String::from("aaaaaazz").into_bytes();
	assert_eq!(next_invite_token(&mut start), String::from("aaaaaazz"));
	assert_eq!(next_invite_token(&mut start), String::from("aaaaabaa"));
	assert_eq!(next_invite_token(&mut start), String::from("aaaaabab"));
}

pub struct Game {
	client_handle: ClientHandle,
	client_color: Color,
	other_client_handle: Option<ClientHandle>,
	id: Id,
	server_id: Id,
	game_state: mach::GameState,
	invite_tokens: Vec<String>,
}

pub type ClientHandle = u64;
