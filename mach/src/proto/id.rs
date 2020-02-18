use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Id(i32);

impl PartialEq<ServerId> for Id {
	fn eq(&self, other: &ServerId) -> bool {
		*self == other.0
	}
}

impl PartialEq<ClientId> for Id {
	fn eq(&self, other: &ClientId) -> bool {
		*self == other.0
	}
}

impl Id {
	pub fn new(id: i32) -> Self {
		Self(id)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerId(Id);

impl ServerId {
	pub fn new(id: Id) -> Self {
		assert!(id.0 < 0);
		Self(id)
	}

	pub fn as_id(self) -> Id {
		self.0
	}
}

impl PartialEq<Id> for ServerId {
	fn eq(&self, other: &Id) -> bool {
		self.0 == *other
	}
}

impl Deref for ServerId {
	type Target = Id;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientId(Id);

impl ClientId {
	pub fn new(id: Id) -> Self {
		assert!(id.0 > 0);
		Self(id)
	}

	pub fn as_id(self) -> Id {
		self.0
	}
}

impl PartialEq<Id> for ClientId {
	fn eq(&self, other: &Id) -> bool {
		self.0 == *other
	}
}

impl Deref for ClientId {
	type Target = Id;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
