use std::io;

use Argdata;
use Integer;
use ReadError;
use Value;

pub struct IntValue<T: Copy> {
	value: T
}

pub fn int<T: Copy>(value: T) -> IntValue<T> {
	IntValue{ value }
}

impl<T: Copy> IntValue<T> where {
	pub fn value(&self) -> T {
		self.value
	}
}

impl<T> Argdata for IntValue<T> where
	T: Copy,
	Integer<'static>: From<T>
{
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(Integer::from(self.value)))
	}

	fn serialized_length(&self) -> usize {
		Integer::from(self.value).serialized_length() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[5])?;
		Integer::from(self.value).serialize(writer)
	}
}
