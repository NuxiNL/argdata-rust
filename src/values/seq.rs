use container::Container;
use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataValue;
use ReadError;
use Seq;
use Value;

pub struct SeqValue<T> {
	items: T,
	length: usize,
}

pub fn seq<T>(items: T) -> SeqValue<T> where
	T: Container,
	<T as Container>::Item: Argdata
{
	let mut length = 1;
	for i in 0..items.len() {
		let a = items.get(i).unwrap();
		length += subfield_length(a.serialized_length());
	}
	SeqValue{ items, length }
}

impl<T> SeqValue<T> where
	T: Container,
	<T as Container>::Item: Argdata
{
	pub fn items(&self) -> &T {
		&self.items
	}
	pub fn into_items(self) -> T {
		self.items
	}
}

impl<T> Argdata for SeqValue<T> where
	T: Container,
	<T as Container>::Item: Argdata
{

	fn read<'b>(&'b self) -> Result<Value<'b>, ReadError> {
		Ok(Value::Seq(self))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[7])?;
		for i in 0..self.items.len() {
			let a = self.items.get(i).unwrap();
			write_subfield_length(a.serialized_length(), writer)?;
			a.serialize(writer)?;
		}
		Ok(())
	}
}

impl<T> Seq for SeqValue<T> where
	T: Container,
	<T as Container>::Item: Argdata
{
	fn iter_seq_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<ArgdataValue<'b>, ReadError>>
	{
		self.items.get(*cookie).map(|a| {
			*cookie += 1;
			Ok(ArgdataValue::Reference(a))
		})
	}
}
