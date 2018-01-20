use std::io;

use Argdata;
use ReadError;
use Timespec;
use Value;

pub struct Timestamp(pub Timespec);

impl Argdata for Timestamp {
	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		Ok(Value::Timestamp(self.0))
	}

	fn serialized_length(&self) -> usize {
		i128_serialized_length(self.nanoseconds()) + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[9])?;
		let nsec = self.nanoseconds();
		let mut n = i128_serialized_length(nsec);
		while n != 0 {
			n -= 1;
			let byte = (nsec >> n * 8) as u8;
			writer.write_all(&[byte])?;
		}
		Ok(())
	}
}

impl Timestamp {
	fn nanoseconds(&self) -> i128 {
		self.0.sec as i128 + self.0.nsec as i128 * 1_000_000_000
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
