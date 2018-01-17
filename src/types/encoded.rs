use byteorder::{ByteOrder, BigEndian};
use std::time::SystemTime;
use std;

use Argdata;
use IntValue;
use NotRead;
use ReadError;
use Type;

pub struct Encoded<'a>(pub &'a [u8]);

impl<'a> Argdata<'a> for Encoded<'a> {

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
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_binary(&'a self) -> Result<&'a [u8], NotRead> {
		match self.0.split_first() {
			Some((&8, data)) => Ok(data),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_bool(&'a self) -> Result<bool, NotRead> {
		match self.0.split_first() {
			Some((&2, data)) if data == &[]  => Ok(false),
			Some((&2, data)) if data == &[1] => Ok(true),
			Some((&2, _)) => Err(NotRead::Error(ReadError::InvalidBoolValue)),
			_ => Err(NotRead::OtherType),
		}
	}

	// TODO: fd (3)

	fn read_float(&'a self) -> Result<f64, NotRead> {
		match self.0.split_first() {
			Some((&4, data)) if data.len() == 8 =>
				Ok(f64::from_bits(BigEndian::read_u64(data))),
			Some((&4, _)) => Err(NotRead::Error(ReadError::InvalidFloatLength)),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_int_value(&'a self) -> Result<IntValue<'a>, NotRead> {
		match self.0.split_first() {
			Some((&5, data)) => Ok(IntValue::from_bigint(data)),
			_ => Err(NotRead::OtherType),
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
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_timestamp(&'a self) -> Result<SystemTime, NotRead> {
		match self.0.split_first() {
			Some((&9, data)) => {

				// 12 bytes is enough for 2**64 seconds in nanoseconds.
				if data.len() > 12 {
					return Err(NotRead::Error(ReadError::TimestampOutOfRange));
				}

				// Extract high 64 bits and lower 32 bits (sign extended).
				let sign = data.len() > 0 && data[0] >= 0x80;
				let high = if sign { !0u64 } else { 0u64 };
				let low = 0u32;
				for (i, &b) in data.iter().enumerate() {
					if i + 4 < data.len() {
						high = high << 8 | b;
					} else {
						low = low << 8 | b;
					}
				}

				let high = high as i64;
				let low = low as u64;

				let high_rem = high % 1_000_000_000;
				high /= 1_000_000_000;
				if high_rem < 0 {
					high_rem += 1_000_000_000;
					high += 1;
				}

				if high < i32::min_value() as i64 || high > i32::max_value() as i64 {
					return Err(NotRead::Error(ReadError::TimestampOutOfRange));
				}

				low += high_rem << 32;
				let nsec = (low % 1_000_000_000) as u32;
				low /= 1_000_000_000;

				// TODO: check overflow
				let sec = low + high << 32;

				Ok(UNIX_EPOCH + Duration::new(secs, nsecs))
			}
			_ => Err(NotRead::OtherType),
		}
	}

	fn serialized_length(&self) -> usize {
		self.0.len()
	}

	fn serialize_into(&self, buf: &mut [u8]) {
		buf.copy_from_slice(self.0);
	}

}
