use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Bool {
	value: bool
}

/// Create an argdata value representing a boolean.
pub fn bool(value: bool) -> Bool{
	Bool{ value }
}

impl Bool {
	pub fn value(&self) -> bool {
		self.value
	}
}

impl<'d> Argdata<'d> for Bool {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Bool(self.value))
	}

	fn serialized_length(&self) -> usize {
		match self.value {
			false => 1,
			true => 2,
		}
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		match self.value {
			false => writer.write_all(&[2]),
			true => writer.write_all(&[2, 1]),
		}
	}
}
