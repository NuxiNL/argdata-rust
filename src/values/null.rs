use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Null;

impl Argdata for Null {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Null)
	}

	fn serialized_length(&self) -> usize {
		0
	}

	fn serialize(&self, _writer: &mut io::Write) -> io::Result<()> {
		Ok(())
	}
}

/// Create an argdata value representing *null*.
pub fn null() -> Null {
	Null
}
