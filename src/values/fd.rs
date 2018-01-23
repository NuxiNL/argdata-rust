use Argdata;
use ReadError;
use Value;
use byteorder::{ByteOrder, BigEndian};
use fd;
use std::io;
use std::os::raw::c_int;

/// Create an argdata value representing a file descriptor of this process.
pub fn process_fd(fd: c_int) -> fd::Fd {
	fd::Fd(fd)
}

/// Create an argdata value representing a file descriptor attached to the data.
pub fn encoded_fd<T: fd::ConvertFd>(raw: u32, convert_fd: T) -> fd::EncodedFd<T> {
	fd::EncodedFd::new(raw, convert_fd)
}

/// Create an argdata value representing an invalid file descriptor.
pub fn invalid_fd() -> fd::InvalidFd {
	fd::InvalidFd
}

impl<'d> Argdata<'d> for fd::Fd {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Fd(encoded_fd(self.0 as u32, &fd::Identity)))
	}

	fn serialized_length(&self) -> usize {
		5
	}

	fn serialize(&self, writer: &mut io::Write, fd_map: Option<&mut fd::FdMapping>) -> io::Result<()> {
		encoded_fd(self.0 as u32, fd::Identity).serialize(writer, fd_map)
	}
}

impl<'d, T: fd::ConvertFd> Argdata<'d> for fd::EncodedFd<T> {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Fd(encoded_fd(self.raw, &self.convert_fd)))
	}

	fn serialized_length(&self) -> usize {
		5
	}

	fn serialize(&self, writer: &mut io::Write, fd_map: Option<&mut fd::FdMapping>) -> io::Result<()> {
		let raw: u32 = if let Some(fd_map) = fd_map {
			self.convert_fd.convert_fd(self.raw).ok().map_or(!0, |fd| fd_map.map(fd))
		} else {
			self.raw
		};
		let mut buf = [3, 0, 0, 0, 0];
		BigEndian::write_u32(&mut buf[1..], raw);
		writer.write_all(&buf)
	}
}

impl<'d> Argdata<'d> for fd::InvalidFd {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Fd(encoded_fd(!0, &fd::NoConvert)))
	}

	fn serialized_length(&self) -> usize {
		5
	}

	fn serialize(&self, writer: &mut io::Write, _: Option<&mut fd::FdMapping>) -> io::Result<()> {
		writer.write_all(&[5, 0xFF, 0xFF, 0xFF, 0xFF])
	}
}
