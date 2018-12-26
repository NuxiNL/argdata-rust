use crate::{fd, Argdata, ReadError, StrValue, Value};
use std::io;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Str<'d> {
	value: &'d str,
}

/// Create an argdata value representing a string.
pub fn str<'d>(value: &'d str) -> Str<'d> {
	Str { value }
}

impl<'d> Str<'d> {
	pub fn str(&self) -> &'d str {
		self.value
	}
}

impl<'d> Argdata<'d> for Str<'d> {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Str(StrValue::from_str(self.str())))
	}

	fn serialized_length(&self) -> usize {
		self.str().len() + 2
	}

	fn serialize(&self, writer: &mut io::Write, _: Option<&mut fd::FdMapping>) -> io::Result<()> {
		writer.write_all(&[8])?;
		writer.write_all(self.str().as_bytes())?;
		writer.write_all(&[0])?;
		Ok(())
	}
}

#[test]
fn str_serialize_test() {
	let s = str("blah");
	assert_eq!(s.serialized_length(), 6);
	let mut buf = Vec::new();
	s.serialize(&mut buf, None).unwrap();
	assert_eq!(&buf, b"\x08blah\x00");
}
