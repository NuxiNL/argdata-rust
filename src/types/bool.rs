use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Bool(pub bool);

impl Argdata for Bool {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Bool(self.0))
	}

	fn serialized_length(&self) -> usize {
		match self.0 {
			false => 1,
			true => 2,
		}
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		match self.0 {
			false => writer.write_all(&[2]),
			true => writer.write_all(&[2, 1]),
		}
	}
}
