use ArgdataRef;
use ReadError;

/// An iterator, iterating over an argdata sequence.
#[derive(Copy, Clone)]
pub struct SeqIterator<'a, 'd: 'a> {
	seq: &'a (SeqIterable<'d> + 'a),
	cookie: usize,
}

/// Something that can be iterated over using a [`SeqIterator`].
pub trait SeqIterable<'d>: Sync {
	/// Iterate to the next element, returning None if the end is reached.
	///
	/// **Don't use this method directly.**
	/// Instead, get a [`SeqIterator`] (using [`::Argdata::read_seq`]), and
	/// use it as any other [`Iterator`].
	///
	/// # For implementors
	///
	/// Use `cookie` to keep track of where the iterator is, and modify it in
	/// this method to advance the iterator to the next element. The value of
	/// the `cookie` is implementation-specific. It might for example be the
	/// index into a vector, or the byte-offset into an encoded argdata value.
	/// The initial value is also implementation specific, so provide a method
	/// (such as [`Argdata.read_seq`]) which provides users with a properly
	/// initialized [`SeqIterator`].
	///
	/// # Panics
	///
	/// May panic if you give it an invalid `cookie`.
	fn iter_seq_next<'a>(
		&'a self,
		cookie: &mut usize,
	) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>>
	where
		'd: 'a;
}

impl<'a, 'd: 'a> SeqIterator<'a, 'd> {
	/// Create a new iterator.
	///
	/// This should only be used in implementations of [`SeqIterable`].
	///
	/// To get a seq iterator over Argdata use [`::Argdata::read_seq`].
	pub fn new(seq: &'a (SeqIterable<'d> + 'a), cookie: usize) -> Self {
		SeqIterator { seq, cookie }
	}
}

impl<'a, 'd: 'a> Iterator for SeqIterator<'a, 'd> {
	type Item = Result<ArgdataRef<'a, 'd>, ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.seq.iter_seq_next(&mut self.cookie)
	}
}

impl std::fmt::Debug for SeqIterator<'_, '_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "SeqIterator(.., {})", self.cookie)
	}
}
