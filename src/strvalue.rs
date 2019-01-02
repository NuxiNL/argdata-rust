use std::ffi::{CStr, FromBytesWithNulError};
use std::fmt::Write;
use std::str::Utf8Error;

/// Represents a string value.
///
/// The string might or might not be zero terminated, and might or might not be
/// valid UTF-8. The accessor functions will check these properties if needed.
#[derive(Clone, Copy)]
pub struct StrValue<'d> {
	inner: Inner<'d>,
}

#[derive(Clone, Copy)]
enum Inner<'d> {
	BytesWithoutNul(&'d [u8]),
	BytesWithNul(&'d [u8]),
	Str(&'d str),
	CStr(&'d CStr),
}

impl<'d> StrValue<'d> {
	/// Create a StrValue referring to raw bytes, which do not include a nul-terminator.
	pub fn from_bytes_without_nul(bytes: &'d [u8]) -> StrValue<'d> {
		StrValue {
			inner: Inner::BytesWithoutNul(bytes),
		}
	}

	/// Create a StrValue referring to raw bytes, which include a nul-terminator.
	///
	/// Panics if the last byte is not a nul-terminator.
	pub fn from_bytes_with_nul(bytes: &'d [u8]) -> StrValue<'d> {
		assert!(bytes.last() == Some(&0));
		StrValue {
			inner: Inner::BytesWithNul(bytes),
		}
	}

	/// Create a StrValue referring to a non-zero terminated UTF-8 `str`.
	pub fn from_str(s: &'d str) -> StrValue<'d> {
		StrValue {
			inner: Inner::Str(s),
		}
	}

	/// Create a StrValue referring to a zero-terminated C string.
	pub fn from_cstr(s: &'d CStr) -> StrValue<'d> {
		StrValue {
			inner: Inner::CStr(s),
		}
	}

	/// Get the raw bytes of the string, without zero terminmator.
	///
	/// Always works, regardless of what type of string the value refers to.
	pub fn as_bytes(&self) -> &'d [u8] {
		match self.inner {
			Inner::BytesWithoutNul(v) => v,
			Inner::BytesWithNul(v) => &v[..v.len() - 1],
			Inner::Str(v) => v.as_bytes(),
			Inner::CStr(v) => v.to_bytes(),
		}
	}

	/// Get the value as `&str`.
	///
	/// Unless the StrValue refers to a `&str`, it is checked for valid UTF-8.
	pub fn as_str(&self) -> Result<&'d str, Utf8Error> {
		match self.inner {
			Inner::Str(v) => Ok(v),
			_ => std::str::from_utf8(self.as_bytes()),
		}
	}

	/// Get the value as (nul-terminated) C string.
	///
	/// This only works when the StrValue:
	///
	///   - refers to a `&Cstr`,
	///   - refers to raw bytes including a nul terminator, but not containing
	///     any other nul bytes (this is checked), or
	///   - refers to an empty string.
	///
	/// Otherwise, it fails.
	///
	/// Note that this should always work for values resulting from properly
	/// encoded argdata, as long as the value doesn't contain any embedded nul
	/// bytes.
	pub fn as_cstr(&self) -> Result<&'d CStr, FromBytesWithNulError> {
		let bytes = match self.inner {
			Inner::CStr(v) => return Ok(v),
			Inner::BytesWithNul(v) => v,
			Inner::Str("") | Inner::BytesWithoutNul(b"") => &[0],
			_ => &[], // Will trigger a missing-nul-terminator error.
		};
		CStr::from_bytes_with_nul(bytes)
	}
}

impl<'d> std::fmt::Debug for StrValue<'d> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.inner {
			Inner::BytesWithoutNul(data) => {
				write!(f, "\"")?;
				for byte in data.iter().flat_map(|&b| std::ascii::escape_default(b)) {
					f.write_char(byte as char)?;
				}
				write!(f, "\" (without NUL terminator)")
			}
			Inner::BytesWithNul(data) => {
				write!(f, "\"")?;
				for byte in data[..data.len() - 1].iter().flat_map(|&b| std::ascii::escape_default(b)) {
					f.write_char(byte as char)?;
				}
				write!(f, "\" (with NUL terminator)")
			}
			Inner::Str(data) => {
				write!(f, "{:?} (checked UTF-8)", data)
			}
			Inner::CStr(data) => {
				write!(f, "{:?} (with NUL terminator, without embedded NULs)", data)
			}
		}
	}
}
