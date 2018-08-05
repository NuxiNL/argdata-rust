use ArgdataRef;
use ReadError;

/// An argdata value representing a map.
pub trait Map<'d> {
	/// Iterate to the next key-value pair, returning None if the end is reached.
	///
	/// You probably want to use `iter_map` instead, which gives you a normal Iterator.
	///
	/// `cookie` should be 0 for the first element, and is modified by this method to refer to the
	/// next key-value pair on each call. The value of the `cookie` is implementation-specific. It
	/// might for example be the index into a vector, or the byte-offset into an encoded argdata
	/// value.
	///
	/// Might panic if you give it an invalid `cookie`.
	fn iter_map_next<'a>(
		&'a self,
		cookie: &mut usize,
	) -> Option<Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>>
	where
		'd: 'a;
}

impl<'a, 'd: 'a> Map<'d> + 'a {
	/// Get an iterator to the key-value pairs of the map.
	pub fn iter_map(&'a self) -> MapIterator<'a, 'd> {
		MapIterator {
			map: self,
			cookie: 0,
		}
	}
}

/// An iterator of a map, returned by `Map::iter_map()`.
pub struct MapIterator<'a, 'd: 'a> {
	map: &'a (Map<'d> + 'a),
	cookie: usize,
}

impl<'a, 'd: 'a> Iterator for MapIterator<'a, 'd> {
	type Item = Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.map.iter_map_next(&mut self.cookie)
	}
}
