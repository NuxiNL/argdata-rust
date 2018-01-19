use Argdata;
use ReadError;
use Timespec;
use Value;

#[derive(Debug)]
pub struct Timestamp(pub Timespec);

impl Argdata for Timestamp {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Timestamp(self.0))
	}

	fn serialized_length(&self) -> usize {
		unimplemented!()
	}

	fn serialize_into(&self, _buf: &mut [u8]) {
		unimplemented!()
	}
}
