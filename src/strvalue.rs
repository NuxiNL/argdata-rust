use std::ffi::{CStr, FromBytesWithNulError};
use std::str::Utf8Error;
use std;

pub struct StrValue<'d> {
	inner: Inner<'d>
}

enum Inner<'d> {
	BytesWithoutNul(&'d [u8]),
	BytesWithNul(&'d [u8]),
	Str(&'d str),
	CStr(&'d CStr),
}

impl<'d> StrValue<'d> {

	pub fn from_bytes_without_nul(bytes: &'d [u8]) -> StrValue<'d> {
		StrValue{ inner: Inner::BytesWithoutNul(bytes) }
	}

	pub fn from_bytes_with_nul(bytes: &'d [u8]) -> StrValue<'d> {
		assert!(bytes.last() == Some(&0));
		StrValue{ inner: Inner::BytesWithNul(bytes) }
	}

	pub fn from_str(s: &'d str) -> StrValue<'d> {
		StrValue{ inner: Inner::Str(s) }
	}

	pub fn from_cstr(s: &'d CStr) -> StrValue<'d> {
		StrValue{ inner: Inner::CStr(s) }
	}

	pub fn as_bytes(&self) -> &'d [u8] {
		match self.inner {
			Inner::BytesWithoutNul(v) => v,
			Inner::BytesWithNul(v) => &v[..v.len()-1],
			Inner::Str(v) => v.as_bytes(),
			Inner::CStr(v) => v.to_bytes(),
		}
	}

	pub fn as_str(&self) -> Result<&'d str, Utf8Error> {
		match self.inner {
			Inner::Str(v) => Ok(v),
			_ => std::str::from_utf8(self.as_bytes())
		}
	}

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
