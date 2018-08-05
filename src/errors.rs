use std;
use std::error::Error;
use std::fmt::Display;
use std::str::Utf8Error;

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

impl Error for ReadError {
	fn description(&self) -> &str {
		#[cfg_attr(rustfmt, rustfmt_skip)]
		match self {
			ReadError::InvalidTag(_)         => "Invalid argdata tag",
			ReadError::MissingNullTerminator => "Argdata contains a string without nul terminator",
			ReadError::InvalidUtf8           => "Argdata contains invalid UTF-8",
			ReadError::InvalidBoolValue      => "Argdata contains an invalid boolean value",
			ReadError::InvalidFloatLength    => "Argdata contains floating point data of invalid length",
			ReadError::InvalidFdLength       => "Argdata contains file descriptor data of invalid length",
			ReadError::TimestampOutOfRange   => "Argdata contains a timestamp which is out of the accepted range",
			ReadError::InvalidSubfield       => "Argdata has an incomplete subfield",
			ReadError::InvalidKeyValuePair   => "Argdata map has an incomplete key-value pair",
			ReadError::InvalidFdNumber(_)    => "Argdata contains a file descriptor that doesn't exist",
		}
	}
}

impl Display for ReadError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.description())?;
		match self {
			ReadError::InvalidTag(x) => write!(f, " (0x{:02X})", x),
			ReadError::InvalidFdNumber(x) => write!(f, " ({})", *x as i32),
			_ => Ok(()),
		}
	}
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

impl From<Utf8Error> for ReadError {
	fn from(_: Utf8Error) -> ReadError {
		ReadError::InvalidUtf8
	}
}

impl From<Utf8Error> for NotRead {
	fn from(_: Utf8Error) -> NotRead {
		NotRead::Error(ReadError::InvalidUtf8)
	}
}
