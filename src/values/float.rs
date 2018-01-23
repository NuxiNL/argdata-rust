use Argdata;
use ReadError;
use Value;
use byteorder::{ByteOrder, BigEndian};
use fd;
use std::io;

pub struct Float{
	value: f64
}

/// Create an argdata value representing a 64-bit floating point value.
pub fn float<T: Into<f64>>(value: T) -> Float {
	Float{ value: value.into() }
}

impl Float {
	pub fn value(&self) -> f64 {
		self.value
	}
}

impl<'d> Argdata<'d> for Float {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Float(self.value))
	}

	fn serialized_length(&self) -> usize {
		9
	}

	fn serialize(&self, writer: &mut io::Write, _: Option<&mut fd::FdMapping>) -> io::Result<()> {
		let mut buf = [0u8; 8];
		BigEndian::write_f64(&mut buf, self.value);
		writer.write_all(&[5])?;
		writer.write_all(&buf)?;
		Ok(())
	}
}
