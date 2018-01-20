use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataValue;
use ReadError;
use Seq;
use Value;

pub struct SeqSlice<'a> {
	items: &'a [&'a (Argdata + 'a)],
	length: usize,
}

impl<'a> SeqSlice<'a> {
	pub fn new(items: &'a [&'a (Argdata + 'a)]) -> Self {
		let mut length = 1;
		for a in items {
			length += subfield_length(a.serialized_length());
		}
		SeqSlice{ items, length }
	}
}

impl<'b> Argdata for SeqSlice<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Seq(self))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[7])?;
		for a in self.items {
			write_subfield_length(a.serialized_length(), writer)?;
			a.serialize(writer)?;
		}
		Ok(())
	}
}

impl<'a> Seq for SeqSlice<'a> {
	fn iter_seq_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<ArgdataValue<'b>, ReadError>>
	{
		self.items.get(*cookie).map(|&a| {
			*cookie += 1;
			Ok(ArgdataValue::Reference(a))
		})
	}
}
