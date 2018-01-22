use std::io;

use Argdata;
use Integer;
use ReadError;
use Value;

pub struct Int<T> {
	value: T
}

/// Create an argdata value representing an integer (of fixed width, e.g. `i32`).
pub fn int<T>(value: T) -> Int<T> where
	T: Copy,
	Integer<'static>: From<T>,
{
	Int{ value }
}

impl<T: Copy> Int<T> where {
	pub fn value(&self) -> T {
		self.value
	}
}

impl<'d, T> Argdata<'d> for Int<T> where
	T: Copy,
	Integer<'static>: From<T>
{
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
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
