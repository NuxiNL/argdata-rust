use Argdata;
use ReadError;
use Timespec;
use Value;
use byteorder::{ByteOrder, BigEndian};
use fd;
use std::io;

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

impl<'d> Argdata<'d> for Timestamp{
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
		Ok(Value::Timestamp(self.value))
	}

	fn serialized_length(&self) -> usize {
		i128_serialized_length(self.nanoseconds()) + 1
	}

	// TODO: Test.
	fn serialize(&self, writer: &mut io::Write, _: Option<&mut fd::FdMapping>) -> io::Result<()> {
		writer.write_all(&[9])?;
		let nsec = self.nanoseconds();
		let n = i128_serialized_length(nsec);
		if n > 0 {
			let mut buf = [0u8; 12];
			BigEndian::write_int128(&mut buf, nsec, n);
			writer.write_all(&buf[..n])?;
		}
		Ok(())
	}
}

impl Timestamp {
	fn nanoseconds(&self) -> i128 {
		i128::from(self.value.sec) * 1_000_000_000 + i128::from(self.value.nsec)
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

#[test]
fn timestamp_serialize_test() {
	for &(timespec, serialized) in &[
		(Timespec{ sec:  0, nsec:           0 }, &b"\x09"[..]),
		(Timespec{ sec:  0, nsec:           1 }, &b"\x09\x01"[..]),
		(Timespec{ sec: -1, nsec: 999_999_999 }, &b"\x09\xFF"[..]),
		(Timespec{ sec: 10, nsec:           0 }, &b"\x09\x02\x54\x0B\xE4\x00"[..]),
	] {
		let value = timestamp(timespec);
		let mut buf = Vec::new();
		value.serialize(&mut buf, None).unwrap();
		assert_eq!(&buf, &serialized);
		assert_eq!(value.serialized_length(), buf.len());
	}
}
