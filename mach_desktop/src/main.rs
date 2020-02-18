use std::{io::Write, net::TcpStream};

use tungstenite::protocol::{Message, WebSocket};

use mach::{game::*, proto::*};

fn setup_logging() -> Result<(), ()> {
	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!("[{}][{}] {}", record.target(), record.level(), message))
		})
		.level(log::LevelFilter::Warn)
		.level_for("mach_desktop", log::LevelFilter::Trace)
		.level_for("mach", log::LevelFilter::Trace)
		.chain(std::io::stderr())
		.apply()
		.map_err(|_| ())
}

fn main() {
	setup_logging().expect("Failed to setup logger");

	let mut client = match Client::new() {
		Ok(client) => client,
		Err(_e) => {
			log::error!("Connection failed");
			return;
		}
	};
	client.init().unwrap();
	let game_id = client.create_game();
	let invite_token = client.get_invite_token(game_id);
	println!("Got invite token '{}'", invite_token);
	let game_state = client.get_game_state(game_id);
	let mut line = String::new();
	render_game(&game_state);
	loop {
		print!("move> ");
		std::io::stdout().flush().unwrap();
		std::io::stdin().read_line(&mut line).unwrap();
		let (move_start, move_end) = match parse_input(&line) {
			Ok(t) => t,
			Err(_) => {
				println!("The move entered was not valid. Enter the start of the move and the finish of the move separated by a space.");
				continue;
			}
		};
		line.clear();
		client.game_move(game_id, move_start, move_end);
		let game_state = client.get_game_state(game_id);
		render_game(&game_state);
	}
}

fn parse_input(line: &str) -> Result<(BoardIndex, BoardIndex), ()> {
	let line = line.trim();
	let mut split = line.split_whitespace();
	let first = parse_board_index(split.next().ok_or(())?)?;
	let second = parse_board_index(split.next().ok_or(())?)?;
	Ok((first, second))
}

fn parse_board_index(input: &str) -> Result<BoardIndex, ()> {
	if input.len() != 2 {
		return Err(());
	};
	let column = match input.as_bytes()[0] {
		b'a' => Column::A,
		b'b' => Column::B,
		b'c' => Column::C,
		b'd' => Column::D,
		b'e' => Column::E,
		b'f' => Column::F,
		b'g' => Column::G,
		b'h' => Column::H,
		_ => return Err(()),
	};
	let row = match input.as_bytes()[1] {
		b'1' => Row::R1,
		b'2' => Row::R2,
		b'3' => Row::R3,
		b'4' => Row::R4,
		b'5' => Row::R5,
		b'6' => Row::R6,
		b'7' => Row::R7,
		b'8' => Row::R8,
		_ => return Err(()),
	};
	Ok(BoardIndex::new(column, row))
}

fn render_game(game_state: &GameState) {
	println!();
	for row in 0..8 {
		let row = Row::from(row);
		print!("\t");
		for column in 0..8 {
			let column = Column::from(column);
			let (c, color) = match game_state.board.get_board_index(BoardIndex::new(column, row)) {
				Some(GamePiece {
					piece: Piece::Pawn,
					color,
				}) => ('p', color),
				Some(GamePiece {
					piece: Piece::Bishop,
					color,
				}) => ('b', color),
				Some(GamePiece {
					piece: Piece::Knight,
					color,
				}) => ('n', color),
				Some(GamePiece {
					piece: Piece::Rook,
					color,
				}) => ('r', color),
				Some(GamePiece {
					piece: Piece::Queen,
					color,
				}) => ('q', color),
				Some(GamePiece {
					piece: Piece::King,
					color,
				}) => ('k', color),
				None => (' ', Color::White),
			};
			let color_start = match color {
				Color::White => "\u{001b}[34m",
				Color::Black => "\u{001b}[31m",
			};
			print!("{}{}\u{001b}[0m", color_start, c);
		}
		println!();
	}
	println!();
}

pub struct Client {
	ws_stream: WebSocket<TcpStream>,
	id_tracker: i32,
}

impl Client {
	pub fn new() -> Result<Client, ()> {
		let stream =
			TcpStream::connect("127.0.0.1:8099").map_err(|e| log::error!("Failed to connect to server: {}", e))?;
		let (ws_stream, _response) = tungstenite::client::client("ws://127.0.0.1:8099", stream)
			.map_err(|e| log::error!("Failed to create websocket connection: {}", e))?;
		Ok(Client {
			ws_stream,
			id_tracker: 1,
		})
	}

	pub fn read_message(&mut self) -> Result<MachMessage, ()> {
		match self.ws_stream.read_message() {
			Ok(Message::Text(text)) => {
				let deserialized: MachMessage = json::from_str(&text).map_err(|_e| {
					eprintln!("Got invalid JSON in message: '{}'", text);
				})?;
				log::trace!("Got mach message: {:?}", deserialized);
				Ok(deserialized)
			}
			Ok(m) => {
				eprintln!("Got unexpected message type: {:?}", m);
				Err(())
			}
			Err(e) => {
				eprintln!("Websocket error: {}", e);
				Err(())
			}
		}
	}

	pub fn init(&mut self) -> Result<(), ()> {
		let message = self.read_message()?;
		let versions = match message {
			MachMessage::Handshake(handshake) => handshake.versions,
			m => {
				eprintln!("Got unexpected message while waiting for server handshake: {:?}", m);
				return Err(());
			}
		};
		let version = *versions.last().unwrap();

		self.ws_stream
			.write_message(Message::Text(
				json::to_string(&MachMessage::HandshakeOk(HandshakeOk { version })).unwrap(),
			))
			.unwrap();

		Ok(())
	}

	pub fn create_game(&mut self) -> ServerId {
		let id = self.next_id();
		self.ws_stream
			.write_message(Message::Text(
				json::to_string(&MachMessage::CreateGameRequest(CreateGameRequest {
					id: *id,
					color: Color::White,
				}))
				.unwrap(),
			))
			.unwrap();
		let message = self.read_message().unwrap();
		match message {
			MachMessage::CreateGameResponse(res) => {
				assert_eq!(res.id, id);
				res.game_id
			}
			m => {
				log::error!("Got unexpected message while waiting for create game response: {:?}", m);
				panic!()
			}
		}
	}

	pub fn get_invite_token(&mut self, game_id: ServerId) -> String {
		let id = self.next_id();
		let message = MachMessage::GetInviteTokenRequest(GetInviteTokenRequest { id: *id, game_id });
		self.ws_stream
			.write_message(Message::Text(json::to_string(&message).unwrap()))
			.unwrap();
		let message = self.read_message().unwrap();
		match message {
			MachMessage::GetInviteTokenResponse(res) => {
				assert_eq!(res.id, id);
				res.invite_token
			}
			m => {
				log::error!(
					"Got unexpected message while waiting for invite token response: {:?}",
					m
				);
				panic!()
			}
		}
	}

	pub fn get_game_state(&mut self, game_id: ServerId) -> GameState {
		let id = self.next_id();
		let message = MachMessage::GetGameStateRequest(GetGameStateRequest { id: *id, game_id });
		self.ws_stream
			.write_message(Message::Text(json::to_string(&message).unwrap()))
			.unwrap();
		let message = self.read_message().unwrap();
		match message {
			MachMessage::GetGameStateResponse(res) => {
				assert_eq!(res.id, id);
				res.game_state
			}
			m => {
				log::error!("Got unexpected message while waiting for game state response: {:?}", m);
				panic!()
			}
		}
	}

	pub fn game_move(&mut self, game_id: ServerId, move_start: BoardIndex, move_end: BoardIndex) {
		let id = self.next_id();
		let message = MachMessage::GameMoveRequest(GameMoveRequest {
			id: *id,
			game_id,
			move_start,
			move_end,
		});
		self.ws_stream
			.write_message(Message::Text(json::to_string(&message).unwrap()))
			.unwrap();
		let message = self.read_message().unwrap();
		match message {
			MachMessage::GameMoveResponse(res) => {
				assert_eq!(res.id, id);
				if res.success {
					println!("Moved successfully");
				} else {
					println!("Piece move failed");
				}
			}
			m => {
				log::error!("Got unexpected message while waiting for game state response: {:?}", m);
				panic!()
			}
		}
	}

	pub fn next_id(&mut self) -> ClientId {
		let id_tracker = self.id_tracker;
		self.id_tracker += 1;
		ClientId::new(Id::new(id_tracker))
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		self.ws_stream.write_message(Message::Close(None)).unwrap();
	}
}
