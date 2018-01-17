use Argdata;
use IntValue;
use ReadError;
use Value;

pub struct BigInt<'a>(pub &'a [u8]);

impl<'a> Argdata<'a> for BigInt<'a> {
	fn read(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from_bigint(self.0)))
	}

	fn serialized_length(&self) -> usize {
		self.0.len() + 1
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		buf[0] = 5;
		buf[1..].copy_from_slice(self.0);
	}
}

pub struct Int<T>(pub T);

impl<'a, T> Argdata<'a> for Int<T>
	where
		T: Copy,
		IntValue<'a>: From<T>
{
	fn read(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from(self.0)))
	}

	fn serialized_length(&self) -> usize {
		unimplemented!()
	}

	fn serialize_into(&self, _buf: &mut [u8]) {
		unimplemented!()
	}
}
