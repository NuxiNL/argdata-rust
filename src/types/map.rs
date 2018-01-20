use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataValue;
use ReadError;
use Map;
use Value;

pub struct MapSlice<'a> {
	items: &'a [(&'a (Argdata + 'a), &'a (Argdata + 'a))],
	length: usize,
}

impl<'a> MapSlice<'a> {
	pub fn new(items: &'a [(&'a (Argdata + 'a), &'a (Argdata + 'a))]) -> Self {
		let mut length = 1;
		for &(k, v) in items {
			length += subfield_length(k.serialized_length());
			length += subfield_length(v.serialized_length());
		}
		MapSlice{ items, length }
	}
}

impl<'b> Argdata for MapSlice<'b> {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Map(self))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[6])?;
		for &(k, v) in self.items {
			write_subfield_length(k.serialized_length(), writer)?;
			k.serialize(writer)?;
			write_subfield_length(v.serialized_length(), writer)?;
			v.serialize(writer)?;
		}
		Ok(())
	}
}

impl<'a> Map for MapSlice<'a> {
	fn iter_map_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<(ArgdataValue<'b>, ArgdataValue<'b>), ReadError>>
	{
		self.items.get(*cookie).map(|&(k, v)| {
			*cookie += 1;
			Ok((
				ArgdataValue::Reference(k),
				ArgdataValue::Reference(v)
			))
		})
	}
}
