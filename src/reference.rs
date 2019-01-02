use crate::{fd, values, Argdata};
use std::ops::Deref;

/// A reference to an argdata value.
/// Either a substring of an encoded argdata value, or just a `&Argdata`.
pub struct ArgdataRef<'a, 'd: 'a> {
	inner: Inner<'a, 'd>,
}

impl<'a, 'd: 'a> ArgdataRef<'a, 'd> {
	/// Create an ArgdataRef that refers to a substring of an encoded argdata value.
	pub fn encoded(bytes: &'d [u8], convert_fd: &'a (dyn fd::ConvertFd + 'a)) -> ArgdataRef<'a, 'd> {
		ArgdataRef {
			inner: Inner::Encoded(crate::encoded_with_fds(bytes, convert_fd)),
		}
	}

	/// Create an ArgdataRef that simply refers to something that implements Argdata.
	pub fn reference(value: &'a (dyn Argdata<'d> + 'a)) -> ArgdataRef<'a, 'd> {
		ArgdataRef {
			inner: Inner::Reference(value),
		}
	}
}

enum Inner<'a, 'd: 'a> {
	Encoded(values::EncodedArgdata<'d, &'a dyn fd::ConvertFd>),
	Reference(&'a dyn Argdata<'d>),
}

impl<'a, 'd: 'a> Deref for ArgdataRef<'a, 'd> {
	type Target = dyn Argdata<'d> + 'a;
	fn deref(&self) -> &Self::Target {
		match self.inner {
			Inner::Encoded(ref argdata) => argdata,
			Inner::Reference(argdata) => argdata,
		}
	}
}
