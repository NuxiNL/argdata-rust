use ArgdataRef;
use ReadError;

/// An argdata value representing a sequence.
pub trait Seq {
	/// Iterate to the next element, returning None if the end is reached.
	///
	/// You probably want to use `iter_seq` instead, which gives you a normal Iterator.
	///
	/// `cookie` should be 0 for the first element, and is modified by this method to refer to the
	/// next element on each call. The value of the `cookie` is implementation-specific. It might
	/// for example be the index into a vector, or the byte-offset into an encoded argdata value.
	///
	/// Might panic if you give it an invalid `cookie`.
	fn iter_seq_next<'a>(&'a self, cookie: &mut usize) ->
		Option<Result<ArgdataRef<'a>, ReadError>>;
}

impl<'a> Seq + 'a {
	/// Get an iterator to the elements of the sequence.
	pub fn iter_seq(&'a self) -> SeqIterator<'a> {
		SeqIterator{
			seq: self,
			cookie: 0
		}
	}
}

/// An iterator of a sequence, returned by `Seq::iter_seq()`.
pub struct SeqIterator<'a> {
	seq: &'a (Seq + 'a),
	cookie: usize,
}

impl<'a> Iterator for SeqIterator<'a> {
	type Item = Result<ArgdataRef<'a>, ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.seq.iter_seq_next(&mut self.cookie)
	}
}
