use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Str<'d> {
	value: &'d str
}

/// Create an argdata value representing a string.
pub fn str<'d>(value: &'d str) -> Str<'d> {
	Str{ value }
}

impl<'d> Str<'d> {
	pub fn str(&self) -> &'d str {
		self.value
	}
}

impl<'d> Argdata<'d> for Str<'d> {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
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
