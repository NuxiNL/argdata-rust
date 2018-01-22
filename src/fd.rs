/// A file descriptor.
pub struct Fd(pub u32);

/// An error indicating that the encoded fd number is invalid:
/// It doesn't refer to any file descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFd;

/// Something that can convert encoded fd numbers to actual Fds.
pub trait ConvertFd {
	/// Converts an encoded fd number to an actual Fd.
	fn convert_fd(&self, raw: u32) -> Result<Fd, InvalidFd>;
}

/// The identity mapping: Convert numbers to Fds with the exact same number.
pub struct Identity;

/// Don't provide any access to Fds. Every conversion will fail.
pub struct NoFds;

/// Convert encoded fd numbers to Fds using the given function.
pub struct ConvertFdFn<F: Fn(u32) -> Result<Fd, InvalidFd>>(pub F);

impl ConvertFd for Identity {
	fn convert_fd(&self, fd: u32) -> Result<Fd, InvalidFd> { Ok(Fd(fd)) }
}

impl ConvertFd for NoFds {
	fn convert_fd(&self, _: u32) -> Result<Fd, InvalidFd> { Err(InvalidFd) }
}

impl<F> ConvertFd for ConvertFdFn<F> where F: Fn(u32) -> Result<Fd, InvalidFd> {
	fn convert_fd(&self, fd: u32) -> Result<Fd, InvalidFd> { self.0(fd) }
}

impl<'a, T> ConvertFd for &'a T where T: ConvertFd + 'a + ?Sized {
	fn convert_fd(&self, fd: u32) -> Result<Fd, InvalidFd> {
		(*self).convert_fd(fd)
	}
}

/// A file descriptor from an (encoded) argdata value that's not yet converted.
#[derive(Clone, Copy)]
pub struct EncodedFd<'a> {
	raw: u32,
	convert_fd: &'a ConvertFd,
}

impl<'a> EncodedFd<'a> {
	/// Create an EncodedFd that will convert `raw` to an Fd using `convert_fd`.
	pub fn new(raw: u32, convert_fd: &'a ConvertFd) -> EncodedFd<'a> {
		EncodedFd{ raw, convert_fd }
	}

	/// The 32-bit file descriptor number exactly as encoded in the raw argdata.
	pub fn raw_encoded_number(&self) -> u32 {
		self.raw
	}

	/// Converts this to a valid file descriptor, if possible.
	pub fn fd(&self) -> Result<Fd, InvalidFd> {
		self.convert_fd.convert_fd(self.raw)
	}
}
