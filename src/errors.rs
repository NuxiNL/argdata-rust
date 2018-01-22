/// An error while reading argdata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {

	/// The data contained the given tag, which doesn't correspond to any known type.
	InvalidTag(u8),

	/// The data represents a string, but it wasn't null-terminated.
	MissingNullTerminator,

	/// The data represents a string, but it contained invalid UTF-8.
	InvalidUtf8,

	/// The data represents a boolean, but it contained a value other than 'false' or 'true'.
	InvalidBoolValue,

	/// The data represents a float, but wasn't exactly 64 bits.
	InvalidFloatLength,

	/// The data represents a file descriptor, but wasn't exactly 32 bits.
	InvalidFdLength,

	/// The data represents a timestamp that does not fit in a Timespec.
	TimestampOutOfRange,

	/// The data contains a subfield (of a map or seq) with an incomplete or too large length.
	InvalidSubfield,

	/// The data contains a map with an incomplete key-value pair.
	InvalidKeyValuePair,

	/// The data represents a file descriptor that doesn't exist.
	/// (Possibly because there were no file descriptors 'attached' to the argdata value at all.)
	InvalidFdNumber(u32),
}

/// The reason why an `Argdata::read_*()` call didn't return a value, when there was no read error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoFit {

	/// The value is too high or low to fit in the requested type.
	OutOfRange,

	/// The value seems to be of a different type.
	DifferentType,
}

/// The reason why an `Argdata::read_*()` call didn't return a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotRead {

	/// The value couldn't be read, because it wouldn't fit in the requested type.
	/// (Because it the value is of a different type, or isn't big enough.)
	NoFit(NoFit),

	/// The value seems to be of the requested type, but it couldn't be read
	/// because of an error.
	Error(ReadError),
}

impl From<ReadError> for NotRead {
	fn from(e: ReadError) -> NotRead {
		NotRead::Error(e)
	}
}

impl From<NoFit> for NotRead {
	fn from(e: NoFit) -> NotRead {
		NotRead::NoFit(e)
	}
}
