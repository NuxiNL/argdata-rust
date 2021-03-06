use crate::{
	container_traits::Container,
	fd,
	subfield::{subfield_length, write_subfield_length},
	Argdata, ArgdataRef, ReadError, SeqIterable, SeqIterator, Value,
};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Seq<'d, T: 'd> {
	items: &'d T,
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
pub fn seq<'d, T>(items: &'d T) -> Seq<'d, T>
where
	T: Container,
	<T as Container>::Item: Argdata<'d>,
{
	let mut length = 1;
	for i in 0..items.len() {
		let a = items.get(i).unwrap();
		length += subfield_length(a.serialized_length());
	}
	Seq { items, length }
}

impl<'d, T> Seq<'d, T>
where
	T: Container,
	<T as Container>::Item: Argdata<'d>,
{
	pub fn elements(&self) -> &'d T {
		&self.items
	}
}

impl<'d, T> Argdata<'d> for Seq<'d, T>
where
	T: Container,
	<T as Container>::Item: Argdata<'d>,
{
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Seq(SeqIterator::new(self, 0)))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(
		&self,
		writer: &mut dyn io::Write,
		mut fd_map: Option<&mut dyn fd::FdMapping>,
	) -> io::Result<()> {
		writer.write_all(&[7])?;
		for i in 0..self.items.len() {
			let a = self.items.get(i).unwrap();
			write_subfield_length(a.serialized_length(), writer)?;
			a.serialize(writer, fd_map.as_mut().map(|x| *x as _))?;
		}
		Ok(())
	}
}

impl<'d, T> SeqIterable<'d> for Seq<'d, T>
where
	T: Container,
	<T as Container>::Item: Argdata<'d>,
{
	fn iter_seq_next<'a>(
		&'a self,
		cookie: &mut usize,
	) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>>
	where
		'd: 'a,
	{
		self.items.get(*cookie).map(|a| {
			*cookie += 1;
			Ok(ArgdataRef::reference(a))
		})
	}
}
