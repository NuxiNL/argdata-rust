use container_traits::MapContainer;
use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataRef;
use ReadError;
use Value;

pub struct Map<T> {
	items: T,
	length: usize,
}

/// Create an argdata value representing a map.
///
/// Note that the data can be (partially) borrowed or owned, depending on the type of container you
/// provide. Also, both a pair of lists and a list of pairs are acceptable containers for `map()`.
///
/// See [`container_trait::MapContainer`](container_traits/trait.MapContainer.html).
///
/// Examples:
///
///  - `map(vec![(key, val), (key, val)])`
///  - `map(&[])`
///  - `let keys = vec![...]; let values = &[...]; map((keys, values))`
///
pub fn map<T>(items: T) -> Map<T> where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata,
	<T as MapContainer>::Value: Argdata,
{
	let mut length = 1;
	for i in 0..items.len() {
		let (k, v) = items.get(i).unwrap();
		length += subfield_length(k.serialized_length());
		length += subfield_length(v.serialized_length());
	}
	Map{ items, length }
}

impl<T> Map<T> where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata,
	<T as MapContainer>::Value: Argdata,
{
	pub fn items(&self) -> &T {
		&self.items
	}
	pub fn into_items(self) -> T {
		self.items
	}
}

impl<T> Argdata for Map<T> where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata,
	<T as MapContainer>::Value: Argdata,
{
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Map(self))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[6])?;
		for i in 0..self.items.len() {
			let (k, v) = self.items.get(i).unwrap();
			write_subfield_length(k.serialized_length(), writer)?;
			k.serialize(writer)?;
			write_subfield_length(v.serialized_length(), writer)?;
			v.serialize(writer)?;
		}
		Ok(())
	}
}

impl<T> ::Map for Map<T> where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata,
	<T as MapContainer>::Value: Argdata,
{
	fn iter_map_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<(ArgdataRef<'b>, ArgdataRef<'b>), ReadError>>
	{
		self.items.get(*cookie).map(|(k, v)| {
			*cookie += 1;
			Ok((
				ArgdataRef::reference(k),
				ArgdataRef::reference(v)
			))
		})
	}
}
