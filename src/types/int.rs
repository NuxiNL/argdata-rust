use std::io;

use Argdata;
use IntValue;
use ReadError;
use Value;

/// A big-endian 2's-complement signed arbitrary length integer.
///
/// Use IntValue::from(BigInt) for easy access to the integral value.
pub struct BigInt<'a>(pub &'a [u8]);

impl<'b> Argdata for BigInt<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from(BigInt(self.0))))
	}

	fn serialized_length(&self) -> usize {
		self.0.len() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[5])?;
		writer.write_all(self.0)?;
		Ok(())
	}
}

pub struct Int<T>(pub T);

impl<T> Argdata for Int<T>
	where
		T: Copy,
		IntValue<'static>: From<T>
{
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from(self.0)))
	}

	fn serialized_length(&self) -> usize {
		IntValue::from(self.0).serialized_length() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[5])?;
		IntValue::from(self.0).serialize(writer)
	}
}
