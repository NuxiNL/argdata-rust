use Argdata;
use ReadError;
use Value;

#[derive(Debug)]
pub struct Null;

impl<'a> Argdata<'a> for Null {
	fn read(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Null)
	}

	fn serialized_length(&self) -> usize {
		0
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		assert_eq!(buf.len(), 0);
	}
}
