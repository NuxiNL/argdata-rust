use std::borrow::Borrow;
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Binary<T: Borrow<[u8]>> {
	value: T
}

/// Create an argdata value representing a binary blob.
///
/// Note that the data can be either owned or borrowed, depending on the type of container you
/// provide. For example: binary(vec![1, 2]) will own the bytes, and binary(&[1, 2]) will borrow.
pub fn binary<T: Borrow<[u8]>>(value: T) -> Binary<T> {
	Binary{ value }
}

impl<T: Borrow<[u8]>> Binary<T> {
	pub fn bytes(&self) -> &[u8] {
		self.value.borrow()
	}
	pub fn into_value(self) -> T {
		self.value
	}
}

impl<T: Borrow<[u8]>> Argdata for Binary<T> {
	fn read<'b>(&'b self) -> Result<Value<'b>, ReadError> {
		Ok(Value::Binary(self.bytes()))
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[1])?;
		writer.write_all(self.bytes())?;
		Ok(())
	}
}
