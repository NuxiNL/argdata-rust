use std::os::raw::c_int;

/// A file descriptor of the current process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fd(pub c_int);

/// A file descriptor in some argdata.
pub struct EncodedFd<T> {
	pub(crate) raw: u32,
	pub(crate) convert_fd: T,
}

/// An error indicating that the encoded fd number is invalid:
/// It doesn't refer to any file descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFd;

/// Something that can convert encoded fd numbers to actual Fds.
pub trait ConvertFd {
	/// Converts an encoded fd number to an actual Fd.
	fn convert_fd(&self, raw: u32) -> Result<Fd, InvalidFd>;
}

/// The identity conversion: Convert numbers to Fds with the exact same number.
pub struct Identity;

/// Don't provide any access to Fds. Every conversion will fail.
///
/// Use this if the encoded numbers (if any) do not correspond to any real file
/// descriptors of this process.
pub struct NoConvert;

/// Convert encoded fd numbers to Fds using the given function.
pub struct ConvertFdFn<F: Fn(u32) -> Result<Fd, InvalidFd>>(pub F);

impl<T: ConvertFd> EncodedFd<T> {
	/// Create an EncodedFd that will convert `raw` to an `Fd` using `convert_fd`.
	pub fn new(raw: u32, convert_fd: T) -> EncodedFd<T> {
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

impl ConvertFd for Identity {
	fn convert_fd(&self, fd: u32) -> Result<Fd, InvalidFd> { Ok(Fd(fd as c_int)) }
}

impl ConvertFd for NoConvert {
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

/// Something that can convert actual Fds to encoded fd numbers.
pub trait FdMapping {
	fn map(&mut self, fd: Fd) -> u32;
}

/// Will number the encoded fds sequentially, storing the actual Fds in the
/// vector at the index of the encoded fd.
///
/// No duplicates will be stored, as (indexes of) existing Fds in the vector
/// are (re)used.
impl FdMapping for Vec<Fd> {
	fn map(&mut self, new_fd: Fd) -> u32 {
		for (i, fd) in self.iter().enumerate() {
			if fd.0 == new_fd.0 {
				return i as u32;
			}
		}
		let new_i = self.len() as u32;
		self.push(new_fd);
		new_i
	}
}
