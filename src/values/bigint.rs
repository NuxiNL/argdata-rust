use std::borrow::Borrow;
use std::io;

use Argdata;
use Integer;
use ReadError;
use Value;

/// A big-endian 2's-complement signed arbitrary length integer.
///
/// Use Integer::from(BigIntValue) for access to the integral value.
// TODO: check if Integer::from still works
pub struct BigIntValue<T: Borrow<[u8]>> {
	value: T
}

pub fn bigint<T: Borrow<[u8]>>(value: T) -> BigIntValue<T> {
	BigIntValue{ value }
}

impl<T: Borrow<[u8]>> BigIntValue<T> {
	pub fn bytes(&self) -> &[u8] {
		self.value.borrow()
	}
	pub fn value(&self) -> &T {
		&self.value
	}
	pub fn into_value(self) -> T {
		self.value
	}
}

impl<T: Borrow<[u8]>> Argdata for BigIntValue<T> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(Integer::from_bigint(self.bytes())))
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[5])?;
		writer.write_all(self.bytes())?;
		Ok(())
	}
}
