use byteorder::{BigEndian, ByteOrder};
use fd;
use std::io;
use Argdata;
use ReadError;
use Timespec;
use Value;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Timestamp {
	value: Timespec,
}

/// Create an argdata value representing a point in time.
pub fn timestamp(value: Timespec) -> Timestamp {
	Timestamp { value }
}

impl Timestamp {
	pub fn value(&self) -> Timespec {
		self.value
	}
}

impl<'d> Argdata<'d> for Timestamp {
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		Ok(Value::Timestamp(self.value))
	}

	fn serialized_length(&self) -> usize {
		i128_serialized_length(self.nanoseconds()) + 1
	}

	fn serialize(&self, writer: &mut io::Write, _: Option<&mut fd::FdMapping>) -> io::Result<()> {
		writer.write_all(&[9])?;
		let nsec = self.nanoseconds();
		let n = i128_serialized_length(nsec);
		if n > 0 {
			let mut buf = [0u8; 12];
			BigEndian::write_i32(&mut buf[0..4], (nsec >> 64) as i32);
			BigEndian::write_u64(&mut buf[4..12], nsec as u64);
			writer.write_all(&buf[12 - n..])?;
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
	#[cfg_attr(rustfmt, rustfmt_skip)]
	for &(timespec, serialized) in &[
		(Timespec { sec:  0, nsec: 0           }, &b"\x09"[..]),
		(Timespec { sec:  0, nsec: 1           }, &b"\x09\x01"[..]),
		(Timespec { sec: -1, nsec: 999_999_999 }, &b"\x09\xFF"[..]),
		(Timespec { sec: 10, nsec: 0           }, &b"\x09\x02\x54\x0B\xE4\x00"[..]),
		(Timespec { sec: 80911113678783, nsec: 24503210 }, &b"\x09\x11\x22\x33\x44\x55\x66\x77\x88\x99\xAA"[..]),
	] {
		let value = timestamp(timespec);
		let mut buf = Vec::new();
		value.serialize(&mut buf, None).unwrap();
		assert_eq!(&buf, &serialized);
		assert_eq!(value.serialized_length(), buf.len());
	}
}
