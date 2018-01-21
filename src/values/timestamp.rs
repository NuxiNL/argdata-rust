use byteorder::{ByteOrder, BigEndian};
use std::io;

use Argdata;
use ReadError;
use Timespec;
use Value;

pub struct Timestamp {
	value: Timespec
}

/// Create an argdata value representing a point in time.
pub fn timestamp(value: Timespec) -> Timestamp {
	Timestamp{ value }
}

impl Timestamp{
	pub fn value(&self) -> Timespec {
		self.value
	}
}

impl Argdata for Timestamp{
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Timestamp(self.value))
	}

	fn serialized_length(&self) -> usize {
		i128_serialized_length(self.nanoseconds()) + 1
	}

	// TODO: Test.
	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[9])?;
		let nsec = self.nanoseconds();
		let n = i128_serialized_length(nsec);
		let mut buf = [0u8; 12];
		BigEndian::write_int128(&mut buf, nsec, n);
		writer.write_all(&buf)?;
		Ok(())
	}
}

impl Timestamp{
	fn nanoseconds(&self) -> i128 {
		self.value.sec as i128 + self.value.nsec as i128 * 1_000_000_000
	}
}

fn i128_serialized_length(v: i128) -> usize {
	if v == 0 {
		0
	} else if v > 0 {
		((128 - v.leading_zeros()) / 8 + 1) as usize
	} else {
		((128 - (!v).leading_zeros()) / 8 + 1) as usize
	}
}
