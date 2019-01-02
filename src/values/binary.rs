use crate::{fd, Argdata, ReadError, Value};
use std::io;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Binary<'d> {
	value: &'d [u8],
}

/// Create an argdata value representing a binary blob.
pub fn binary<'d>(value: &'d [u8]) -> Binary<'d> {
	Binary { value }
}

impl<'d> Binary<'d> {
	pub fn bytes(&self) -> &'d [u8] {
		self.value
	}
}

impl<'d> Argdata<'d> for Binary<'d> {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Binary(self.bytes()))
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len() + 1
	}

	fn serialize(&self, writer: &mut dyn io::Write, _: Option<&mut dyn fd::FdMapping>) -> io::Result<()> {
		writer.write_all(&[1])?;
		writer.write_all(self.bytes())?;
		Ok(())
	}
}
