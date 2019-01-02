use crate::ReadError;
use std::io;

pub fn read_subfield(data: &[u8]) -> (Option<Result<&[u8], ReadError>>, usize) {
	if data.is_empty() {
		return (None, 0);
	}

	// Decode field size
	let mut len_bytes: usize = 0;
	let mut len: usize = 0;
	loop {
		let byte = match data.get(len_bytes) {
			Some(&v) => v,
			None => return (Some(Err(ReadError::InvalidSubfield)), data.len()),
		};
		len_bytes += 1;
		if len > usize::max_value() >> 7 {
			return (Some(Err(ReadError::InvalidSubfield)), data.len());
		}
		len = len << 7 | (byte & 0x7F) as usize;
		if byte >= 0x80 {
			break;
		}
	}

	// Get len bytes after the encoded length.
	if len > data[len_bytes..].len() {
		return (Some(Err(ReadError::InvalidSubfield)), data.len());
	}
	let field = &data[len_bytes..][..len];

	(Some(Ok(field)), len_bytes + len)
}

pub fn subfield_length_length(length: usize) -> usize {
	let n_bits = 0usize.count_zeros() - (length | 1).leading_zeros();
	((n_bits + 6) / 7) as usize
}

pub fn subfield_length(length: usize) -> usize {
	subfield_length_length(length) + length
}

pub fn write_subfield_length(length: usize, writer: &mut dyn io::Write) -> io::Result<()> {
	let mut n = subfield_length_length(length);
	while n != 0 {
		n -= 1;
		let mut byte = (length >> (7 * n)) as u8;
		if n == 0 {
			byte |= 0x80;
		} else {
			byte &= 0x7F;
		}
		writer.write_all(&[byte])?;
	}
	Ok(())
}

#[test]
fn test_subfield_length() {
	assert_eq!(subfield_length(0), 1 + 0);
	assert_eq!(subfield_length(1), 1 + 1);
	assert_eq!(subfield_length(0x7F), 1 + 0x7F);
	assert_eq!(subfield_length(0x80), 2 + 0x80);
	assert_eq!(subfield_length(0x3FFF), 2 + 0x3FFF);
	assert_eq!(subfield_length(0x4000), 3 + 0x4000);
}

#[test]
fn test_write_subfield_length() {
	let mut v = Vec::new();
	write_subfield_length(0, &mut v).unwrap();
	assert_eq!(v, &[0x80]);

	let mut v = Vec::new();
	write_subfield_length(1, &mut v).unwrap();
	assert_eq!(v, &[0x81]);

	let mut v = Vec::new();
	write_subfield_length(0x80, &mut v).unwrap();
	assert_eq!(v, &[0x01, 0x80]);

	let mut v = Vec::new();
	write_subfield_length(0x3FFF, &mut v).unwrap();
	assert_eq!(v, &[0x7F, 0xFF]);

	let mut v = Vec::new();
	write_subfield_length(0x4000, &mut v).unwrap();
	assert_eq!(v, &[0x01, 0x00, 0x80]);
}
