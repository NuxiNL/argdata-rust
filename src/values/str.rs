use std::borrow::Borrow;
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Str<T: Borrow<str>> {
	value: T
}

/// Create an argdata value representing a string.
///
/// Note that the data can be either owned or borrowed, depending on the type of container you
/// provide.
/// For example: str("asdf".to_string()) will own the string, and binary(s.as_str()) will borrow.
pub fn str<T: Borrow<str>>(value: T) -> Str<T> {
	Str{ value }
}

impl<T: Borrow<str>> Str<T> {
	pub fn str(&self) -> &str {
		self.value.borrow()
	}
	pub fn value(&self) -> &T {
		&self.value
	}
	pub fn into_value(self) -> T {
		self.value
	}
}

impl<T: Borrow<str>> Argdata for Str<T> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Str(self.str()))
	}

	fn serialized_length(&self) -> usize {
		self.str().len() + 2
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[8])?;
		writer.write_all(self.str().as_bytes())?;
		writer.write_all(&[0])?;
		Ok(())
	}
}
