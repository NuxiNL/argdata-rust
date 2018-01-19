use Argdata;
use ReadError;
use Value;

#[derive(Debug)]
pub struct Str<'a>(pub &'a str);

impl<'b> Argdata for Str<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Str(self.0))
	}

	fn serialized_length(&self) -> usize {
		self.0.len() + 2
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		let strlen = self.0.len();
		assert_eq!(buf.len(), strlen + 2);
		buf[0] = 8;
		buf[1..strlen+1].copy_from_slice(self.0.as_bytes());
		buf[strlen+1] = 0;
	}
}
