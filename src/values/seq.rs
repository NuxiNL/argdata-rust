use container_traits::Container;
use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataRef;
use ReadError;
use Value;

pub struct Seq<T> {
	items: T,
	length: usize,
}

/// Create an argdata value representing a sequence.
///
/// Note that the data can be (partially) borrowed or owned, depending on the type of container you
/// provide.
///
/// See [`container_trait::Container`](container_traits/trait.Container.html).
///
/// Examples:
///
///  - `seq(vec![a, b, c])`
///  - `seq(&[])`
///  - `seq(Rc::new([int(1), int(2)])`
///
pub fn seq<T>(items: T) -> Seq<T> where
	T: Container,
	<T as Container>::Item: Argdata
{
	let mut length = 1;
	for i in 0..items.len() {
		let a = items.get(i).unwrap();
		length += subfield_length(a.serialized_length());
	}
	Seq{ items, length }
}

impl<T> Seq<T> where
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

impl<T> Argdata for Seq<T> where
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

impl<T> ::Seq for Seq<T> where
	T: Container,
	<T as Container>::Item: Argdata
{
	fn iter_seq_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<ArgdataRef<'b>, ReadError>>
	{
		self.items.get(*cookie).map(|a| {
			*cookie += 1;
			Ok(ArgdataRef::reference(a))
		})
	}
}
