use Argdata;
use ReadError;
use Value;

#[derive(Debug)]
pub struct Bool(pub bool);

impl<'a> Argdata<'a> for Bool {
	fn read(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Bool(self.0))
	}

	fn serialized_length(&self) -> usize {
		match self.0 {
			true => 2,
			false => 1,
		}
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		match self.0 {
			true => buf.copy_from_slice(&[2, 1]),
			false => buf.copy_from_slice(&[2]),
		}
	}
}
