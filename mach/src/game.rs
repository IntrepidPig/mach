use serde::{Deserialize, Serialize};

pub const SIZE: u32 = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameState {
	pub board: GameBoard,
	pub turn: Color,
}

impl GameState {
	pub fn new() -> Self {
		Self {
			board: GameBoard::new(),
			turn: Color::White,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameBoard {
	/// Row major storage for the board grid. The grid is represented with square A8 at index 0. This
	/// Vec will always be of size (SIZE * SIZE).
	board: Vec<Option<GamePiece>>,
}

impl GameBoard {
	/// Create an empty board
	pub fn new() -> Self {
		Self {
			board: (0..(SIZE * SIZE)).into_iter().map(|_| None).collect(),
		}
	}

	/// Set the board to the standard layout
	pub fn set_standard(&mut self) {
		use self::Color::*;
		use self::Column::*;
		use self::Piece::*;
		use self::Row::*;

		*self.get_board_index_mut(BoardIndex::new(A, R8)) = Some(GamePiece::new(Rook, Black));
		*self.get_board_index_mut(BoardIndex::new(B, R8)) = Some(GamePiece::new(Knight, Black));
		*self.get_board_index_mut(BoardIndex::new(C, R8)) = Some(GamePiece::new(Bishop, Black));
		*self.get_board_index_mut(BoardIndex::new(D, R8)) = Some(GamePiece::new(Queen, Black));
		*self.get_board_index_mut(BoardIndex::new(E, R8)) = Some(GamePiece::new(King, Black));
		*self.get_board_index_mut(BoardIndex::new(F, R8)) = Some(GamePiece::new(Bishop, Black));
		*self.get_board_index_mut(BoardIndex::new(G, R8)) = Some(GamePiece::new(Knight, Black));
		*self.get_board_index_mut(BoardIndex::new(H, R8)) = Some(GamePiece::new(Rook, Black));
		*self.get_board_index_mut(BoardIndex::new(A, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(B, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(C, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(D, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(E, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(F, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(G, R7)) = Some(GamePiece::new(Pawn, Black));
		*self.get_board_index_mut(BoardIndex::new(H, R7)) = Some(GamePiece::new(Pawn, Black));

		*self.get_board_index_mut(BoardIndex::new(A, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(B, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(C, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(D, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(E, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(F, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(G, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(H, R2)) = Some(GamePiece::new(Pawn, White));
		*self.get_board_index_mut(BoardIndex::new(A, R1)) = Some(GamePiece::new(Rook, White));
		*self.get_board_index_mut(BoardIndex::new(B, R1)) = Some(GamePiece::new(Knight, White));
		*self.get_board_index_mut(BoardIndex::new(C, R1)) = Some(GamePiece::new(Bishop, White));
		*self.get_board_index_mut(BoardIndex::new(D, R1)) = Some(GamePiece::new(Queen, White));
		*self.get_board_index_mut(BoardIndex::new(E, R1)) = Some(GamePiece::new(King, White));
		*self.get_board_index_mut(BoardIndex::new(F, R1)) = Some(GamePiece::new(Bishop, White));
		*self.get_board_index_mut(BoardIndex::new(G, R1)) = Some(GamePiece::new(Knight, White));
		*self.get_board_index_mut(BoardIndex::new(H, R1)) = Some(GamePiece::new(Rook, White));

		for row_num in 2..6 {
			let row = Row::from(row_num);
			for col_num in 0..8 {
				let column = Column::from(col_num);
				*self.get_board_index_mut(BoardIndex::new(column, row)) = None;
			}
		}
	}

	/// Get the current value of the cell at the given board index
	pub fn get_board_index(&self, index: BoardIndex) -> Option<GamePiece> {
		self.board[index.to_linear()]
	}

	/// Get a mutable reference to the cell at the given board index
	pub fn get_board_index_mut(&mut self, index: BoardIndex) -> &mut Option<GamePiece> {
		&mut self.board[index.to_linear()]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoardIndex {
	pub column: Column,
	pub row: Row,
}

impl BoardIndex {
	pub fn new(column: Column, row: Row) -> Self {
		Self { column, row }
	}

	pub fn to_linear(self) -> usize {
		let column: u32 = self.column.into();
		let row: u32 = self.row.into();
		(row * SIZE) as usize + column as usize
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Column {
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
}

impl From<u32> for Column {
	fn from(t: u32) -> Self {
		match t {
			0 => Column::A,
			1 => Column::B,
			2 => Column::C,
			3 => Column::D,
			4 => Column::E,
			5 => Column::F,
			6 => Column::G,
			7 => Column::H,
			_ => panic!("Invalid column index: {}", t),
		}
	}
}

impl From<Column> for u32 {
	fn from(t: Column) -> Self {
		match t {
			Column::A => 0,
			Column::B => 1,
			Column::C => 2,
			Column::D => 3,
			Column::E => 4,
			Column::F => 5,
			Column::G => 6,
			Column::H => 7,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Row {
	R1,
	R2,
	R3,
	R4,
	R5,
	R6,
	R7,
	R8,
}

impl From<u32> for Row {
	fn from(t: u32) -> Self {
		match t {
			0 => Row::R1,
			1 => Row::R2,
			2 => Row::R3,
			3 => Row::R4,
			4 => Row::R5,
			5 => Row::R6,
			6 => Row::R7,
			7 => Row::R8,
			_ => panic!("Invalid row index: {}", t),
		}
	}
}

impl From<Row> for u32 {
	fn from(t: Row) -> Self {
		match t {
			Row::R1 => 0,
			Row::R2 => 1,
			Row::R3 => 2,
			Row::R4 => 3,
			Row::R5 => 4,
			Row::R6 => 5,
			Row::R7 => 6,
			Row::R8 => 7,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GamePiece {
	pub piece: Piece,
	pub color: Color,
}

impl GamePiece {
	pub fn new(piece: Piece, color: Color) -> Self {
		Self { piece, color }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Piece {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
	Black,
	White,
}
