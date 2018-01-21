use std::borrow::Borrow;
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct StrValue<T: Borrow<str>> {
	value: T
}

pub fn str<T: Borrow<str>>(value: T) -> StrValue<T> {
	StrValue{ value }
}

impl<T: Borrow<str>> StrValue<T> {
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

impl<T: Borrow<str>> Argdata for StrValue<T> {
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
