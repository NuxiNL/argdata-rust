use std::fmt;

use Argdata;
use ArgdataValue;
use ReadError;
use Seq;
use Value;

//#[derive(Debug)]
pub struct SeqSlice<'a>(pub &'a [&'a (Argdata + 'a)]);

impl<'b> Argdata for SeqSlice<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Seq(self))
	}

	fn serialized_length(&self) -> usize {
		unimplemented!()
	}

	fn serialize_into(&self, _buf: &mut [u8]) {
		unimplemented!()
	}
}

impl<'a> Seq for SeqSlice<'a> {
	fn iter_seq_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<ArgdataValue<'b>, ReadError>> {
		self.0.get(*cookie).map(|&a| {
			*cookie += 1;
			Ok(ArgdataValue::Reference(a))
		})
	}
}

impl<'a> fmt::Debug for SeqSlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "ARGDATA") // TODO
	}
}
