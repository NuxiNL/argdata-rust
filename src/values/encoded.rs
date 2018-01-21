use byteorder::{ByteOrder, BigEndian};
use fd;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::io;
use std;
use subfield::read_subfield;

use Argdata;
use ArgdataRef;
use Integer;
use Map;
use NoFit;
use NotRead;
use ReadError;
use Seq;
use Timespec;
use Type;

pub struct EncodedArgdata<T: Borrow<[u8]>, F: fd::ConvertFd> {
	encoded: T,
	convert_fd: F,
}

impl<T: Borrow<[u8]>, F: fd::ConvertFd> EncodedArgdata<T, F> {
	pub fn bytes(&self) -> &[u8] {
		self.encoded.borrow()
	}
	pub fn encoded(&self) -> &T {
		&self.encoded
	}
	pub fn into_encoded(self) -> T {
		self.encoded
	}
}

/// Create an argdata value directly from an encoded argdata buffer.
///
/// Reading file descriptors from this value is disabled.
///
/// The data is not converted, it will be decoded on demand.
///
/// Note that the data can be either owned or borrowed, depending on the type of container you
/// provide. For example: encoded(vec![2, 1]) will own the bytes, and encoded(&[2, 1]) will borrow.
pub fn encoded<T: Borrow<[u8]>>(encoded: T) -> EncodedArgdata<T, fd::NoFds> {
	EncodedArgdata{ encoded, convert_fd: fd::NoFds }
}

/// Create an argdata value directly from an encoded argdata buffer, wich has filedescriptors
/// attached somehow.
///
/// Reading file descriptors will use the provided `convert_fd` object.
///
/// The data is not converted, it will be decoded on demand.
///
/// Note that the data can be either owned or borrowed, depending on the type of container you
/// provide. For example: encoded(vec![2, 1]) will own the bytes, and encoded(&[2, 1]) will borrow.
pub fn encoded_with_fds<T: Borrow<[u8]>, F: fd::ConvertFd>
	(encoded: T, convert_fd: F) -> EncodedArgdata<T, F>
{
	EncodedArgdata{ encoded, convert_fd }
}

impl<T: Borrow<[u8]>, F: fd::ConvertFd> Argdata for EncodedArgdata<T, F> {

	fn get_type(&self) -> Result<Type, ReadError> {
		match self.bytes().first() {
			None => Ok(Type::Null),
			Some(&1) => Ok(Type::Binary),
			Some(&2) => Ok(Type::Bool),
			Some(&3) => Ok(Type::Fd),
			Some(&4) => Ok(Type::Float),
			Some(&5) => Ok(Type::Int),
			Some(&6) => Ok(Type::Map),
			Some(&7) => Ok(Type::Seq),
			Some(&8) => Ok(Type::Str),
			Some(&9) => Ok(Type::Timestamp),
			Some(&tag) => Err(ReadError::InvalidTag(tag)),
		}
	}

	fn read_null(&self) -> Result<(), NotRead> {
		match self.bytes().len() {
			0 => Ok(()),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_binary<'a>(&'a self) -> Result<&'a [u8], NotRead> {
		match self.bytes().split_first() {
			Some((&8, data)) => Ok(data),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_bool(&self) -> Result<bool, NotRead> {
		match self.bytes().split_first() {
			Some((&2, data)) if data == &[]  => Ok(false),
			Some((&2, data)) if data == &[1] => Ok(true),
			Some((&2, _)) => Err(NotRead::Error(ReadError::InvalidBoolValue)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_encoded_fd(&self) -> Result<fd::EncodedFd, NotRead> {
		match self.bytes().split_first() {
			Some((&3, data)) if data.len() == 4 => Ok(fd::EncodedFd::new(
				BigEndian::read_u32(data),
				&self.convert_fd
			)),
			Some((&3, _)) => Err(NotRead::Error(ReadError::InvalidFdLength)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_float(&self) -> Result<f64, NotRead> {
		match self.bytes().split_first() {
			Some((&4, data)) if data.len() == 8 =>
				Ok(f64::from_bits(BigEndian::read_u64(data))),
			Some((&4, _)) => Err(NotRead::Error(ReadError::InvalidFloatLength)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_int_value<'a>(&'a self) -> Result<Integer<'a>, NotRead> {
		match self.bytes().split_first() {
			Some((&5, data)) => Ok(Integer::from_bigint(data)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_map<'a>(&'a self) -> Result<&'a (Map + 'a), NotRead> {
		match self.bytes().first() {
			Some(&6) => Ok(self),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_seq<'a>(&'a self) -> Result<&'a (Seq + 'a), NotRead> {
		match self.bytes().first() {
			Some(&7) => Ok(self),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_str<'a>(&'a self) -> Result<&'a str, NotRead> {
		match self.bytes().split_first() {
			Some((&8, data)) => match data.split_last() {
				Some((&0, str_bytes)) =>
					std::str::from_utf8(str_bytes).map_err(|_|
						NotRead::Error(ReadError::InvalidUtf8)
					),
				_ => Err(NotRead::Error(ReadError::MissingNullTerminator)),
			}
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_timestamp(&self) -> Result<Timespec, NotRead> {
		match self.bytes().split_first() {
			Some((&9, data)) => {

				// 12 bytes is enough for 2**64 seconds in nanoseconds.
				if data.len() > 12 {
					return Err(ReadError::TimestampOutOfRange.into());
				}

				// Read nanoseconds into an integer (128 bits are enough).
				let mut nsec = BigEndian::read_int128(data, data.len());

				// Split seconds and nanoseconds
				let mut sec = nsec / 1_000_000_000;
				nsec %= 1_000_000_000;
				if nsec < 0 {
					nsec += 1_000_000_000;
					sec -= 1;
				}

				// Convert to i64 and i32, if it fits.
				let sec: i64 = TryFrom::try_from(sec).map_err(|_|
					NotRead::Error(ReadError::TimestampOutOfRange)
				)?;
				let nsec = nsec as u32;

				Ok(Timespec{sec, nsec})
			}
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len()
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(self.bytes())
	}

}

impl<T: Borrow<[u8]>, F: fd::ConvertFd> EncodedArgdata<T, F> {
	fn iter_subfield_next<'b>(&'b self, tag: u8, offset: &mut usize) -> Option<Result<ArgdataRef<'b>, ReadError>> {
		if self.bytes().get(0) != Some(&tag) { return None }
		let (result, offset_delta) = read_subfield(&self.bytes()[1 + *offset..]);
		*offset += offset_delta;
		result.map(|r| r.map(|d| ArgdataRef::encoded(d, &self.convert_fd)))
	}
}

impl<T: Borrow<[u8]>, F: fd::ConvertFd> Seq for EncodedArgdata<T, F> {
	fn iter_seq_next<'b>(&'b self, offset: &mut usize) -> Option<Result<ArgdataRef<'b>, ReadError>> {
		self.iter_subfield_next(7, offset)
	}
}

impl<T: Borrow<[u8]>, F: fd::ConvertFd> Map for EncodedArgdata<T, F> {
	fn iter_map_next<'b>(&'b self, offset: &mut usize) -> Option<Result<(ArgdataRef<'b>, ArgdataRef<'b>), ReadError>> {
		let key = match self.iter_subfield_next(6, offset) {
			None => return None,
			Some(Ok(v)) => v,
			Some(Err(e)) => return Some(Err(e)),
		};
		match self.iter_subfield_next(6, offset) {
			None => Some(Err(ReadError::InvalidKeyValuePair)),
			Some(Ok(value)) => Some(Ok((key, value))),
			Some(Err(e)) => Some(Err(e)),
		}
	}
}
