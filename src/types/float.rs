use byteorder::{ByteOrder, BigEndian};

use Argdata;
use ReadError;
use Value;

#[derive(Debug)]
pub struct Float(pub f64);

impl Argdata for Float {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Float(self.0))
	}

	fn serialized_length(&self) -> usize {
		9
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		assert_eq!(buf.len(), 9);
		buf[0] = 4;
		BigEndian::write_f64(&mut buf[1..9], self.0);
	}
}
