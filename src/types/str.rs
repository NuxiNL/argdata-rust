use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Str<'a>(pub &'a str);

impl<'b> Argdata for Str<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Str(self.0))
	}

	fn serialized_length(&self) -> usize {
		self.0.len() + 2
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[8])?;
		writer.write_all(self.0.as_bytes())?;
		writer.write_all(&[0])?;
		Ok(())
	}
}
