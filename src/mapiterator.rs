use ArgdataRef;
use ReadError;

/// An iterator, iterating over an argdata map.
#[derive(Copy, Clone)]
pub struct MapIterator<'a, 'd: 'a> {
	map: &'a (MapIterable<'d> + 'a),
	cookie: usize,
}

/// Something that can be iterated over using a [`MapIterator`].
pub trait MapIterable<'d>: Sync {
	/// Iterate to the next key-value pair, returning None if the end is reached.
	///
	/// **Don't use this method directly.**
	/// Instead, get a [`MapIterator`] (using [`::Argdata::read_map`]), and
	/// use it as any other [`Iterator`].
	///
	/// # For implementors
	///
	/// Use `cookie` to keep track of where the iterator is, and modify it in
	/// this method to advance the iterator to the next element. The value of
	/// the `cookie` is implementation-specific. It might for example be the
	/// index into a vector, or the byte-offset into an encoded argdata value.
	/// The initial value is also implementation specific, so provide a method
	/// (such as [`Argdata.read_map`]) which provides users with a properly
	/// initialized [`MapIterator`].
	///
	/// # Panics
	///
	/// May panic if you give it an invalid `cookie`.
	fn iter_map_next<'a>(
		&'a self,
		cookie: &mut usize,
	) -> Option<Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>>
	where
		'd: 'a;
}

impl<'a, 'd: 'a> MapIterator<'a, 'd> {
	/// Create a new iterator.
	///
	/// This should only be used in implementations of [`MapIterable`].
	///
	/// To get a map iterator over Argdata use [`::Argdata::read_map`].
	pub fn new(map: &'a (MapIterable<'d> + 'a), cookie: usize) -> Self {
		MapIterator { map, cookie }
	}
}

impl<'a, 'd: 'a> Iterator for MapIterator<'a, 'd> {
	type Item = Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.map.iter_map_next(&mut self.cookie)
	}
}
