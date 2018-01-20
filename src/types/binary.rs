use std::io;

use Argdata;
use ReadError;
use Value;

#[derive(Debug)]
pub struct Binary<'a>(pub &'a [u8]);

impl<'a> Argdata for Binary<'a> {
	fn read<'b>(&'b self) -> Result<Value<'b>, ReadError> {
		Ok(Value::Binary(self.0))
	}

	fn serialized_length(&self) -> usize {
		self.0.len() + 1
	}

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()> {
		writer.write_all(&[1])?;
		writer.write_all(self.0)?;
		Ok(())
	}
}

/* TODO
#[derive(Debug)]
pub struct OwnedBinary<'a>(pub Vec<u8>);

impl<'a> OwnedArgdata<'a> for OwnedBinary {
	type Borrowed = Binary<'a>;
	fn borrow(&'a self) -> Binary<'a> {
		Binary(&self.0[..])
	}
}
*/
