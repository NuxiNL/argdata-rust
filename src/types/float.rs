use byteorder::{ByteOrder, BigEndian};
use std::io;

use Argdata;
use ReadError;
use Value;

pub struct Float(pub f64);

impl Argdata for Float {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Float(self.0))
	}

	fn serialized_length(&self) -> usize {
		9
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		let mut buf = [0u8; 8];
		BigEndian::write_f64(&mut buf, self.0);
		writer.write_all(&[5])?;
		writer.write_all(&buf)?;
		Ok(())
	}
}
