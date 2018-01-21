use std::borrow::Borrow;
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct BinaryValue<T: Borrow<[u8]>> {
	value: T
}

pub fn binary<T: Borrow<[u8]>>(value: T) -> BinaryValue<T> {
	BinaryValue{ value }
}

impl<T: Borrow<[u8]>> BinaryValue<T> {
	pub fn bytes(&self) -> &[u8] {
		self.value.borrow()
	}
	pub fn into_value(self) -> T {
		self.value
	}
}

impl<T: Borrow<[u8]>> Argdata for BinaryValue<T> {
	fn read<'b>(&'b self) -> Result<Value<'b>, ReadError> {
		Ok(Value::Binary(self.bytes()))
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[1])?;
		writer.write_all(self.bytes())?;
		Ok(())
	}
}
