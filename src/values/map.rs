use container_traits::MapContainer;
use fd;
use std::io;
use subfield::{subfield_length, write_subfield_length};
use Argdata;
use ArgdataRef;
use ReadError;
use Value;

#[derive(Clone, Copy, Debug)]
pub struct Map<'d, T: 'd> {
	items: &'d T,
	length: usize,
}

/// Create an argdata value representing a map.
///
/// Both a pair of lists and a list of pairs are acceptable containers for `map()`.
/// See [`container_trait::MapContainer`](container_traits/trait.MapContainer.html).
///
/// Examples:
///
///  - `map(&[(key, val), (key, val)])`
///  - `map(&[])`
///  - `let keys = vec![...]; let values = &[...]; map(&(keys, values))`
///
pub fn map<'d, T>(items: &'d T) -> Map<'d, T>
where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata<'d>,
	<T as MapContainer>::Value: Argdata<'d>,
{
	let mut length = 1;
	for i in 0..items.len() {
		let (k, v) = items.get(i).unwrap();
		length += subfield_length(k.serialized_length());
		length += subfield_length(v.serialized_length());
	}
	Map { items, length }
}

impl<'d, T> Map<'d, T>
where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata<'d>,
	<T as MapContainer>::Value: Argdata<'d>,
{
	pub fn elements(&self) -> &'d T {
		self.items
	}
}

impl<'d, T> Argdata<'d> for Map<'d, T>
where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata<'d>,
	<T as MapContainer>::Value: Argdata<'d>,
{
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Map(self))
	}

	fn serialized_length(&self) -> usize {
		self.length
	}

	fn serialize(
		&self,
		writer: &mut io::Write,
		mut fd_map: Option<&mut fd::FdMapping>,
	) -> io::Result<()> {
		writer.write_all(&[6])?;
		#[cfg_attr(rustfmt, rustfmt_skip)]
		for i in 0..self.items.len() {
			let (k, v) = self.items.get(i).unwrap();
			write_subfield_length(k.serialized_length(), writer)?;
			k.serialize(writer, if let Some(ref mut m) = fd_map { Some(*m) } else { None })?;
			write_subfield_length(v.serialized_length(), writer)?;
			v.serialize(writer, if let Some(ref mut m) = fd_map { Some(*m) } else { None })?;
		}
		Ok(())
	}
}

impl<'d, T> ::Map<'d> for Map<'d, T>
where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata<'d>,
	<T as MapContainer>::Value: Argdata<'d>,
{
	fn iter_map_next<'a>(
		&'a self,
		cookie: &mut usize,
	) -> Option<Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>>
	where
		'd: 'a,
	{
		self.items.get(*cookie).map(|(k, v)| {
			*cookie += 1;
			Ok((ArgdataRef::reference(k), ArgdataRef::reference(v)))
		})
	}
}
