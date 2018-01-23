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
use byteorder::{ByteOrder, BigEndian};
use encoded_fd;
use fd;
use std::convert::TryFrom;
use std::io;
use std;
use subfield::read_subfield;

pub struct EncodedArgdata<'d, F> {
	encoded: &'d [u8],
	convert_fd: F,
}

impl<'d, F: fd::ConvertFd> EncodedArgdata<'d, F> {
	pub fn bytes(&self) -> &'d [u8] {
		self.encoded
	}
}

/// Create an argdata value directly from an encoded argdata buffer.
///
/// The data is not converted, it will be decoded on demand.
pub fn encoded<'d>(encoded: &'d [u8]) -> EncodedArgdata<'d, fd::NoConvert> {
	EncodedArgdata{ encoded, convert_fd: fd::NoConvert }
}

/// Create an argdata value directly from an encoded argdata buffer, which has
/// file descriptors attached.
///
/// Reading file descriptors will use the provided `convert_fd` object.
///
/// The data is not converted, it will be decoded on demand.
pub fn encoded_with_fds<'d, F: fd::ConvertFd>
	(encoded: &'d [u8], convert_fd: F) -> EncodedArgdata<'d, F>
{
	EncodedArgdata{ encoded, convert_fd }
}

impl<'d, F: fd::ConvertFd> Argdata<'d> for EncodedArgdata<'d, F> {

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

	fn read_binary(&self) -> Result<&'d [u8], NotRead> {
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

	fn read_encoded_fd<'a>(&'a self) -> Result<fd::EncodedFd<&'a fd::ConvertFd>, NotRead> where 'd: 'a {
		match self.bytes().split_first() {
			Some((&3, data)) if data.len() == 4 => Ok(encoded_fd(
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

	fn read_int_value(&self) -> Result<Integer<'d>, NotRead> {
		match self.bytes().split_first() {
			Some((&5, data)) => Ok(Integer::from_bigint(data)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_map<'a>(&'a self) -> Result<&'a (Map<'d> + 'a), NotRead> where 'd: 'a {
		match self.bytes().first() {
			Some(&6) => Ok(self),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_seq<'a>(&'a self) -> Result<&'a (Seq<'d> + 'a), NotRead> where 'd: 'a {
		match self.bytes().first() {
			Some(&7) => Ok(self),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_str(&self) -> Result<&'d str, NotRead> {
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

	fn serialize(&self, writer: &mut io::Write, fd_map: Option<&mut fd::FdMapping>) -> io::Result<()> {
		if let Some(fd_map) = fd_map {
			match self.get_type() {
				Ok(Type::Map) | Ok(Type::Seq) => {
					let mut last_write_offset = 0;
					let mut offset = 0;
					while let Some(Ok(a)) = self.iter_subfield_next(&mut offset) {
						writer.write_all(&self.bytes()[last_write_offset..offset+1])?;
						last_write_offset = offset + 1;
						a.serialize(writer, Some(fd_map))?;
					}
					Ok(())
				},
				Ok(Type::Fd) => {
					let efd = self.read_encoded_fd().unwrap_or(encoded_fd(!0, &fd::NoConvert));
					efd.serialize(writer, Some(fd_map))
				}
				_ => writer.write_all(self.bytes()),
			}
		} else {
			writer.write_all(self.bytes())
		}
	}

}

impl<'d, F: fd::ConvertFd> EncodedArgdata<'d, F> {
	fn iter_subfield_next<'a>(&'a self, offset: &mut usize) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>> where 'd: 'a {
		let (result, offset_delta) = read_subfield(&self.bytes()[1 + *offset..]);
		*offset += offset_delta;
		result.map(|r| r.map(|d| ArgdataRef::encoded(d, &self.convert_fd)))
	}
}

impl<'d, F: fd::ConvertFd> Seq<'d> for EncodedArgdata<'d, F> {
	fn iter_seq_next<'a>(&'a self, offset: &mut usize) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>> where 'd: 'a {
		if self.bytes().get(0) != Some(&7) { return None }
		self.iter_subfield_next(offset)
	}
}

impl<'d, F: fd::ConvertFd> Map<'d> for EncodedArgdata<'d, F> {
	fn iter_map_next<'a>(&'a self, offset: &mut usize) -> Option<Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>> where 'd: 'a {
		if self.bytes().get(0) != Some(&6) { return None }
		let key = match self.iter_subfield_next(offset) {
			None => return None,
			Some(Ok(v)) => v,
			Some(Err(e)) => return Some(Err(e)),
		};
		match self.iter_subfield_next(offset) {
			None => Some(Err(ReadError::InvalidKeyValuePair)),
			Some(Ok(value)) => Some(Ok((key, value))),
			Some(Err(e)) => Some(Err(e)),
		}
	}
}
