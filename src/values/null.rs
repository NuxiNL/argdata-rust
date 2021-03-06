use crate::{fd, Argdata, ReadError, Value};
use std::io;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Null;

impl<'d> Argdata<'d> for Null {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Null)
	}

	fn serialized_length(&self) -> usize {
		0
	}

	fn serialize(&self, _: &mut dyn io::Write, _: Option<&mut dyn fd::FdMapping>) -> io::Result<()> {
		Ok(())
	}
}

/// Create an argdata value representing *null*.
pub fn null() -> Null {
	Null
}
