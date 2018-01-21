use byteorder::{ByteOrder, BigEndian};
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct FloatValue {
	value: f64
}

pub fn float<T: Into<f64>>(value: T) -> FloatValue {
	FloatValue{ value: value.into() }
}

impl FloatValue {
	pub fn value(&self) -> f64 {
		self.value
	}
}

impl Argdata for FloatValue {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Float(self.value))
	}

	fn serialized_length(&self) -> usize {
		9
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		let mut buf = [0u8; 8];
		BigEndian::write_f64(&mut buf, self.value);
		writer.write_all(&[5])?;
		writer.write_all(&buf)?;
		Ok(())
	}
}
