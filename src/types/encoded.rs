use byteorder::{ByteOrder, BigEndian};
use std::convert::TryFrom;
use std;

use Argdata;
use BigInt;
use IntValue;
use NoFit;
use NotRead;
use ReadError;
use Timespec;
use Type;

pub struct EncodedArgdata<'a>(pub &'a [u8]);

impl<'a> Argdata<'a> for EncodedArgdata<'a> {

	fn get_type(&'a self) -> Result<Type, ReadError> {
		match self.0.first() {
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

	fn read_null(&'a self) -> Result<(), NotRead> {
		match self.0.len() {
			0 => Ok(()),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_binary(&'a self) -> Result<&'a [u8], NotRead> {
		match self.0.split_first() {
			Some((&8, data)) => Ok(data),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_bool(&'a self) -> Result<bool, NotRead> {
		match self.0.split_first() {
			Some((&2, data)) if data == &[]  => Ok(false),
			Some((&2, data)) if data == &[1] => Ok(true),
			Some((&2, _)) => Err(NotRead::Error(ReadError::InvalidBoolValue)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	// TODO: fd (3)

	fn read_float(&'a self) -> Result<f64, NotRead> {
		match self.0.split_first() {
			Some((&4, data)) if data.len() == 8 =>
				Ok(f64::from_bits(BigEndian::read_u64(data))),
			Some((&4, _)) => Err(NotRead::Error(ReadError::InvalidFloatLength)),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_int_value(&'a self) -> Result<IntValue<'a>, NotRead> {
		match self.0.split_first() {
			Some((&5, data)) => Ok(IntValue::from(BigInt(data))),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	// TODO: map (6)
	// TODO: seq (7)

	fn read_str(&'a self) -> Result<&'a str, NotRead> {
		match self.0.split_first() {
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

	fn read_timestamp(&'a self) -> Result<Timespec, NotRead> {
		match self.0.split_first() {
			Some((&9, data)) => {

				// 12 bytes is enough for 2**64 seconds in nanoseconds.
				if data.len() > 12 {
					return Err(ReadError::TimestampOutOfRange.into());
				}

				// Read nanoseconds into an integer (128 bits are enough).
				let sign = data.len() > 0 && data[0] >= 0x80;
				let mut nsec = if sign { -1i128 } else { 1i128 };
				for &b in data {
					nsec = nsec << 8 | (b as i128);
				}

				// Split seconds and nanoseconds
				let mut sec = nsec / 1_000_000_000;
				nsec %= 1_000_000_000;
				if nsec < 0 {
					nsec += 1_000_000_000;
					sec -= 1;
				}

				// Convert to i64 and i32, if it fits.
				let sec: i64 = TryFrom::try_from(sec).map_err(|_| NotRead::Error(ReadError::TimestampOutOfRange))?;
				let nsec = nsec as u32;

				Ok(Timespec{sec, nsec})
			}
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn serialized_length(&self) -> usize {
		self.0.len()
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		buf.copy_from_slice(self.0);
	}

}
