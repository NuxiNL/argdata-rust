use std::io;

use Argdata;
use Integer;
use ReadError;
use Value;

/// A big-endian 2's-complement signed arbitrary length integer.
pub struct BigInt<'d> {
	value: &'d [u8]
}

/// Create an argdata value representing an arbitrary length 2's complement integer.
pub fn bigint<'d>(value: &'d [u8]) -> BigInt<'d> {
	BigInt{ value }
}

impl<'d> BigInt<'d> {
	pub fn bytes(&self) -> &'d [u8] {
		self.value
	}
	pub fn integer(&self) -> Integer<'d> {
		Integer::from_bigint(self.bytes())
	}
}

impl<'d> Argdata<'d> for BigInt<'d> {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
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
