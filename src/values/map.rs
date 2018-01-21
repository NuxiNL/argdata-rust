use container::MapContainer;
use std::io;
use subfield::{subfield_length, write_subfield_length};

use Argdata;
use ArgdataValue;
use ReadError;
use Map;
use Value;

pub struct MapValue<T> {
	items: T,
	length: usize,
}

pub fn map<T>(items: T) -> MapValue<T> where
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
	MapValue{ items, length }
}

impl<T> MapValue<T> where
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

impl<T> Argdata for MapValue<T> where
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

impl<T> Map for MapValue<T> where
	T: MapContainer,
	<T as MapContainer>::Key: Argdata,
	<T as MapContainer>::Value: Argdata,
{
	fn iter_map_next<'b>(&'b self, cookie: &mut usize) ->
		Option<Result<(ArgdataValue<'b>, ArgdataValue<'b>), ReadError>>
	{
		self.items.get(*cookie).map(|(k, v)| {
			*cookie += 1;
			Ok((
				ArgdataValue::Reference(k),
				ArgdataValue::Reference(v)
			))
		})
	}
}
