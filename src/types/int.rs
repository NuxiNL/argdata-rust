use Argdata;
use IntValue;
use ReadError;
use Value;

/// A big-endian 2's-complement signed arbitrary length integer.
///
/// Use IntValue::from(BigInt) for easy access to the integral value.
pub struct BigInt<'a>(pub &'a [u8]);

impl<'b> Argdata for BigInt<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from(BigInt(self.0))))
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

impl<T> Argdata for Int<T>
	where
		T: Copy,
		IntValue<'static>: From<T>
{
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Int(IntValue::from(self.0)))
	}

	fn serialized_length(&self) -> usize {
		unimplemented!()
	}

	fn serialize_into(&self, _buf: &mut [u8]) {
		unimplemented!()
	}
}
