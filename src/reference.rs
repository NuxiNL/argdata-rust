use Argdata;
use fd;
use std::ops::Deref;
use values;

/// A reference to an argdata value.
/// Either a substring of an encoded argdata value, or just a `&Argdata`.
pub struct ArgdataRef<'a> {
	inner: Inner<'a>
}

impl<'a> ArgdataRef<'a> {

	/// Create an ArgdataRef that refers to a substring of an encoded argdata value.
	pub fn encoded(bytes: &'a [u8], convert_fd: &'a (fd::ConvertFd + 'a)) -> ArgdataRef<'a> {
		ArgdataRef{ inner: Inner::Encoded(::encoded_with_fds(bytes, convert_fd)) }
	}

	/// Create an ArgdataRef that simply refers to something that implements Argdata.
	pub fn reference(value: &'a (Argdata + 'a)) -> ArgdataRef<'a> {
		ArgdataRef{ inner: Inner::Reference(value) }
	}
}

pub enum Inner<'a> {
	Encoded(values::EncodedArgdata<&'a [u8], &'a (fd::ConvertFd + 'a)>),
	Reference(&'a (Argdata + 'a)),
}

impl<'a> Deref for ArgdataRef<'a> {
	type Target = Argdata + 'a;
	fn deref(&self) -> &Self::Target {
		match &self.inner {
			&Inner::Encoded(ref argdata) => argdata,
			&Inner::Reference(argdata) => argdata,
		}
	}
}
