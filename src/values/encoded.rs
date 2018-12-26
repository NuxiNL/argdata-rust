use byteorder::{BigEndian, ByteOrder};
use fd;
use fd::EncodedFd;
use std::io;
use subfield::read_subfield;
use Argdata;
use ArgdataRef;
use IntValue;
use MapIterable;
use MapIterator;
use NoFit;
use NotRead;
use ReadError;
use SeqIterable;
use SeqIterator;
use StrValue;
use Timespec;
use Type;

#[derive(Clone, Copy, Debug)]
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
/// The data is not converted. It will be decoded on demand.
pub fn encoded<'d>(encoded: &'d [u8]) -> EncodedArgdata<'d, fd::NoConvert> {
	EncodedArgdata {
		encoded,
		convert_fd: fd::NoConvert,
	}
}

/// Create an argdata value directly from an encoded argdata buffer, which has
/// file descriptors attached.
///
/// Reading file descriptors will use the provided `convert_fd` object.
///
/// The data is not converted. It will be decoded on demand.
pub fn encoded_with_fds<'d, F: fd::ConvertFd>(
	encoded: &'d [u8],
	convert_fd: F,
) -> EncodedArgdata<'d, F> {
	EncodedArgdata {
		encoded,
		convert_fd,
	}
}

impl<'d, F: fd::ConvertFd> Argdata<'d> for EncodedArgdata<'d, F> {
	fn get_type(&self) -> Result<Type, ReadError> {
		match self.bytes().first() {
			None => Ok(Type::Null),
			Some(1) => Ok(Type::Binary),
			Some(2) => Ok(Type::Bool),
			Some(3) => Ok(Type::Fd),
			Some(4) => Ok(Type::Float),
			Some(5) => Ok(Type::Int),
			Some(6) => Ok(Type::Map),
			Some(7) => Ok(Type::Seq),
			Some(8) => Ok(Type::Str),
			Some(9) => Ok(Type::Timestamp),
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
			Some((1, data)) => Ok(data),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_bool(&self) -> Result<bool, NotRead> {
		match self.bytes().split_first() {
			Some((2, [])) => Ok(false),
			Some((2, [1])) => Ok(true),
			Some((2, _)) => Err(NotRead::Error(ReadError::InvalidBoolValue)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_encoded_fd<'a>(&'a self) -> Result<fd::EncodedFd<&'a fd::ConvertFd>, NotRead>
	where
		'd: 'a,
	{
		match self.bytes().split_first() {
			Some((3, data)) if data.len() == 4 => Ok(EncodedFd {
				raw: BigEndian::read_u32(data),
				convert_fd: &self.convert_fd,
			}),
			Some((3, _)) => Err(NotRead::Error(ReadError::InvalidFdLength)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_float(&self) -> Result<f64, NotRead> {
		match self.bytes().split_first() {
			Some((4, data)) if data.len() == 8 => Ok(f64::from_bits(BigEndian::read_u64(data))),
			Some((4, _)) => Err(NotRead::Error(ReadError::InvalidFloatLength)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_int_value(&self) -> Result<IntValue<'d>, NotRead> {
		match self.bytes().split_first() {
			Some((5, data)) => Ok(IntValue::from_bigint(data)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_map<'a>(&'a self) -> Result<MapIterator<'a, 'd>, NotRead>
	where
		'd: 'a,
	{
		match self.bytes().first() {
			Some(6) => Ok(MapIterator::new(self, 1)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_seq<'a>(&'a self) -> Result<SeqIterator<'a, 'd>, NotRead>
	where
		'd: 'a,
	{
		match self.bytes().first() {
			Some(7) => Ok(SeqIterator::new(self, 1)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_str_value(&self) -> Result<StrValue<'d>, NotRead> {
		match self.bytes().split_first() {
			Some((8, data)) if data.last() == Some(&0) => Ok(StrValue::from_bytes_with_nul(data)),
			Some((8, _)) => Err(ReadError::MissingNullTerminator.into()),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_timestamp(&self) -> Result<Timespec, NotRead> {
		match self.bytes().split_first() {
			Some((9, data)) => {
				// 12 bytes is enough for 2**64 seconds in nanoseconds.
				if data.len() > 12 {
					return Err(ReadError::TimestampOutOfRange.into());
				}

				// Read nanoseconds into an integer (128 bits are enough).
				let mut nsec = if data.len() == 0 {
					0
				} else {
					BigEndian::read_int128(data, data.len())
				};

				// Split seconds and nanoseconds
				let mut sec = nsec / 1_000_000_000;
				nsec %= 1_000_000_000;
				if nsec < 0 {
					nsec += 1_000_000_000;
					sec -= 1;
				}

				// Convert to i64 and i32, if it fits.
				// TODO: Replace by TryFrom::try_from(sec) when TryFrom is stabilized.
				if sec as i64 as i128 != sec {
					return Err(ReadError::TimestampOutOfRange.into());
				}
				let sec = sec as i64;
				let nsec = nsec as u32; // Always fits, since it is ∈ [0, 1e9).

				Ok(Timespec { sec, nsec })
			}
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn serialized_length(&self) -> usize {
		self.bytes().len()
	}

	fn serialize(
		&self,
		writer: &mut io::Write,
		fd_map: Option<&mut fd::FdMapping>,
	) -> io::Result<()> {
		if let Some(fd_map) = fd_map {
			match self.get_type() {
				Ok(Type::Map) | Ok(Type::Seq) => {
					let mut last_write_offset = 0;
					let mut offset = 0;
					while let Some(Ok(a)) = self.iter_subfield_next(&mut offset) {
						writer.write_all(&self.bytes()[last_write_offset..offset + 1])?;
						last_write_offset = offset + 1;
						a.serialize(writer, Some(fd_map))?;
					}
					Ok(())
				}
				Ok(Type::Fd) => {
					let efd = self.read_encoded_fd().unwrap_or(EncodedFd {
						raw: !0,
						convert_fd: &fd::NoConvert,
					});
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
	fn iter_subfield_next<'a>(
		&'a self,
		offset: &mut usize,
	) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>>
	where
		'd: 'a,
	{
		let (result, offset_delta) = read_subfield(&self.bytes()[*offset..]);
		*offset += offset_delta;
		result.map(|r| r.map(|d| ArgdataRef::encoded(d, &self.convert_fd)))
	}
}

impl<'d, F: fd::ConvertFd> SeqIterable<'d> for EncodedArgdata<'d, F> {
	fn iter_seq_next<'a>(
		&'a self,
		offset: &mut usize,
	) -> Option<Result<ArgdataRef<'a, 'd>, ReadError>>
	where
		'd: 'a,
	{
		if self.bytes().get(0) != Some(&7) {
			return None;
		}
		self.iter_subfield_next(offset)
	}
}

impl<'d, F: fd::ConvertFd> MapIterable<'d> for EncodedArgdata<'d, F> {
	fn iter_map_next<'a>(
		&'a self,
		offset: &mut usize,
	) -> Option<Result<(ArgdataRef<'a, 'd>, ArgdataRef<'a, 'd>), ReadError>>
	where
		'd: 'a,
	{
		if self.bytes().get(0) != Some(&6) {
			return None;
		}
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

#[test]
fn get_type_test() {
	assert_eq!(encoded(b"").get_type(), Ok(Type::Null));
	assert_eq!(encoded(b"\x01").get_type(), Ok(Type::Binary));
	assert_eq!(encoded(b"\x02").get_type(), Ok(Type::Bool));
	assert_eq!(encoded(b"\x03").get_type(), Ok(Type::Fd));
	assert_eq!(encoded(b"\x04").get_type(), Ok(Type::Float));
	assert_eq!(encoded(b"\x05").get_type(), Ok(Type::Int));
	assert_eq!(encoded(b"\x06").get_type(), Ok(Type::Map));
	assert_eq!(encoded(b"\x07").get_type(), Ok(Type::Seq));
	assert_eq!(encoded(b"\x08").get_type(), Ok(Type::Str));
	assert_eq!(encoded(b"\x09").get_type(), Ok(Type::Timestamp));
	assert_eq!(encoded(b"\x0A").get_type(), Err(ReadError::InvalidTag(10)));
}

#[test]
fn read_null_test() {
	assert_eq!(encoded(b"").read_null(), Ok(()));
	assert_eq!(encoded(b"\x01").read_null(), Err(NoFit::DifferentType.into()));
}

#[test]
fn read_binary_test() {
	assert_eq!(encoded(b"\x01").read_binary(), Ok(&b""[..]));
	assert_eq!(encoded(b"\x01\x00\x11\x22").read_binary(), Ok(&b"\x00\x11\x22"[..]));
	assert_eq!(encoded(b"").read_binary(), Err(NoFit::DifferentType.into()));
	assert_eq!(encoded(b"\x02").read_binary(), Err(NoFit::DifferentType.into()));
}

#[test]
fn read_bool_test() {
	assert_eq!(encoded(b"\x02").read_bool(), Ok(false));
	assert_eq!(encoded(b"\x02\x01").read_bool(), Ok(true));
	assert_eq!(encoded(b"\x02\x01\x01").read_bool(), Err(ReadError::InvalidBoolValue.into()));
	assert_eq!(encoded(b"\x02\x00").read_bool(), Err(ReadError::InvalidBoolValue.into()));
	assert_eq!(encoded(b"\x02\xFF").read_bool(), Err(ReadError::InvalidBoolValue.into()));
	assert_eq!(encoded(b"").read_bool(), Err(NoFit::DifferentType.into()));
	assert_eq!(encoded(b"\x01").read_bool(), Err(NoFit::DifferentType.into()));
}

#[test]
fn read_encoded_fd_test() {
	assert_eq!(encoded(b"\x03\x00\x00\x00\x00").read_encoded_fd().unwrap().raw_encoded_number(), 0);
	assert_eq!(encoded(b"\x03\x00\x00\x01\x23").read_encoded_fd().unwrap().raw_encoded_number(), 0x123);
	assert_eq!(encoded(b"\x03\xFF\xFF\xFF\xFF").read_encoded_fd().unwrap().raw_encoded_number(), !0);
	assert_eq!(encoded(b"\x03\x01").read_encoded_fd().unwrap_err(), ReadError::InvalidFdLength.into());
	assert_eq!(encoded(b"\x03").read_encoded_fd().unwrap_err(), ReadError::InvalidFdLength.into());
	assert_eq!(encoded(b"\x0312345").read_encoded_fd().unwrap_err(), ReadError::InvalidFdLength.into());
	assert_eq!(encoded(b"").read_encoded_fd().unwrap_err(), NoFit::DifferentType.into());
	assert_eq!(encoded(b"\x01").read_encoded_fd().unwrap_err(), NoFit::DifferentType.into());
}

#[test]
fn read_float_test() {
	assert_eq!(encoded(b"\x04\x00\x00\x00\x00\x00\x00\x00\x00").read_float(), Ok(0.0));
	assert_eq!(encoded(b"\x04\x3F\xF8\x00\x00\x00\x00\x00\x00").read_float(), Ok(1.5));
	assert_eq!(encoded(b"\x04\x7F\xF0\x00\x00\x00\x00\x00\x00").read_float(), Ok(std::f64::INFINITY));
	assert!(encoded(b"\x04\xFF\xFF\xFF\xFF\xFF\x00\x00\x00").read_float().unwrap().is_nan());
	assert_eq!(encoded(b"\x04123").read_float(), Err(ReadError::InvalidFloatLength.into()));
	assert_eq!(encoded(b"\x04").read_float(), Err(ReadError::InvalidFloatLength.into()));
	assert_eq!(encoded(b"\x04123456789").read_float(), Err(ReadError::InvalidFloatLength.into()));
	assert_eq!(encoded(b"").read_float(), Err(NoFit::DifferentType.into()));
	assert_eq!(encoded(b"\x01").read_float(), Err(NoFit::DifferentType.into()));
}

#[test]
fn read_int_test() {
	use ArgdataExt;
	assert_eq!(encoded(b"\x05").read_int(), Ok(0));
	assert_eq!(encoded(b"\x05\x01").read_int(), Ok(1));
	assert_eq!(encoded(b"\x05\xFF").read_int(), Ok(-1));
	assert_eq!(encoded(b"\x05\x3F\xF8").read_int::<u16>(), Ok(0x3FF8));
	assert_eq!(encoded(b"\x05\x3F\xF8").read_int::<u8>(), Err(NoFit::OutOfRange.into()));
	assert_eq!(encoded(b"\x05\xFF").read_int::<u8>(), Err(NoFit::OutOfRange.into()));
	assert_eq!(encoded(b"").read_int_value(), Err(NoFit::DifferentType.into()));
	assert_eq!(encoded(b"\x04").read_int_value(), Err(NoFit::DifferentType.into()));
}

// TODO: Tests for maps

// TODO: Tests for sequences

#[test]
fn read_str_test() {
	use ArgdataExt;
	assert_eq!(encoded(b"\x08\x00").read_str(), Ok(""));
	assert_eq!(encoded(b"\x08Hello World!\x00").read_str(), Ok("Hello World!"));
	assert_eq!(encoded(b"\x08\xCE\xB1\xCE\xB2\xCE\xBE\xCE\xB4\x00").read_str(), Ok("αβξδ"));
	assert_eq!(encoded(b"\x08\x80abc\x00").read_str(), Err(ReadError::InvalidUtf8.into()));
	assert_eq!(encoded(b"\x08").read_str(), Err(ReadError::MissingNullTerminator.into()));
	assert_eq!(encoded(b"\x08Hello World!").read_str(), Err(ReadError::MissingNullTerminator.into()));
	assert_eq!(encoded(b"").read_str(), Err(NoFit::DifferentType.into()));
	assert_eq!(encoded(b"\x01").read_str(), Err(NoFit::DifferentType.into()));
}

#[test]
fn read_timestamp_test() {
	assert_eq!(
		encoded(b"\x09").read_timestamp(),
		Ok(Timespec { sec: 0, nsec: 0 })
	);
	assert_eq!(
		encoded(b"\x09\x01").read_timestamp(),
		Ok(Timespec { sec: 0, nsec: 1 })
	);
	assert_eq!(
		encoded(b"\x09\xFF").read_timestamp(),
		Ok(Timespec { sec: -1, nsec: 999_999_999 })
	);
	assert_eq!(
		encoded(b"\x09\x02\x54\x0B\xE4\x00").read_timestamp(),
		Ok(Timespec { sec: 10, nsec: 0 })
	);
	assert_eq!(
		encoded(b"\x09\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA").read_timestamp(),
		Ok(Timespec { sec: 80911113678783, nsec: 24503210 })
	);
	assert_eq!(
		encoded(b"\x09\x80\x00\x00\x00\x00\x00\x00\x00\x00\x01").read_timestamp(),
		Ok(Timespec { sec: -604462909807315, nsec: 412646913 })
	);
	assert_eq!(
		encoded(b"\x08").read_timestamp(),
		Err(NoFit::DifferentType.into()),
	);
}
