use std::borrow::Borrow;
use std::io;

use Argdata;
use Integer;
use ReadError;
use Value;

/// A big-endian 2's-complement signed arbitrary length integer.
pub struct BigInt<T: Borrow<[u8]>> {
	value: T
}

/// Create an argdata value representing an arbitrary length 2's complement integer.
///
/// Note that the data can be either owned or borrowed, depending on the type of container you
/// provide. For example: bigint(vec![1, 2]) will own the bytes, and bigint(&[1, 2]) will borrow.
pub fn bigint<T: Borrow<[u8]>>(value: T) -> BigInt<T> {
	BigInt{ value }
}

impl<T: Borrow<[u8]>> BigInt<T> {
	pub fn bytes(&self) -> &[u8] {
		self.value.borrow()
	}
	pub fn value(&self) -> &T {
		&self.value
	}
	pub fn into_value(self) -> T {
		self.value
	}
	pub fn integer<'a>(&'a self) -> Integer<'a> {
		Integer::from_bigint(self.bytes())
	}
}

impl<T: Borrow<[u8]>> Argdata for BigInt<T> {
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
