use std::io;

use Argdata;
use ReadError;
use Value;

pub struct BoolValue {
	value: bool
}

pub fn bool(value: bool) -> BoolValue {
	BoolValue{ value }
}

impl BoolValue {
	pub fn value(&self) -> bool {
		self.value
	}
}

impl Argdata for BoolValue {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
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
