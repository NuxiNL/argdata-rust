use std::num::TryFromIntError;
use std::convert::TryFrom;

use BigInt;

#[derive(PartialEq, Eq)]
pub struct IntValue<'a> {
	inner: Inner<'a>
}

#[derive(PartialEq, Eq)]
enum Inner<'a> {
	Unsigned(u64),
	Signed(i64), // For negative numbers only.
	Big(&'a [u8]), // Big-endian 2's-complement signed integer of arbitrary length, that doesn't fit into a i64 or u64.
}

impl<'a> From<BigInt<'a>> for IntValue<'a> {
	fn from(BigInt(mut data): BigInt<'a>) -> IntValue<'a> {
		// If it is positive and fits in an u64, will make an Inner::Unsigned.
		// If it is negative and fits in an i64, will make an Inner::Signed.
		// Otherwise, will make an Inner::Big with the borrowed data, with
		// unnecessary leading zeros/ones stripped.

		let sign = *data.get(0).unwrap_or(&0) >= 0x80;

		while
			data.get(0) == Some(if sign { &0xFF } else { &0 }) &&
			(*data.get(1).unwrap_or(&0) >= 0x80) == sign
		{
			data = &data[1..]
		}

		if data.len() > 9 || (data.len() == 9 && data[0] != 0) {
			return IntValue{ inner: Inner::Big(data) };
		}

		let mut value: u64 = if sign { !0 } else { 0 };

		for &byte in data {
			value = value << 8 | byte as u64;
		}

		if sign {
			IntValue{ inner: Inner::Signed(value as i64) }
		} else {
			IntValue{ inner: Inner::Unsigned(value) }
		}
	}
}

macro_rules! impl_s {
	($t:ty) => {
		impl<'a> From<$t> for IntValue<'a> {
			fn from(value: $t) -> IntValue<'a> {
				if value < 0 {
					IntValue{ inner: Inner::Signed(value.into()) }
				} else {
					IntValue{ inner: Inner::Unsigned(value as u64) }
				}
			}
		}
		impl<'a> TryFrom<IntValue<'a>> for $t {
			type Error = TryFromIntError;
			fn try_from(value: IntValue<'a>) -> Result<$t, Self::Error> {
				match value.inner {
					Inner::Unsigned(v) => Ok(TryFrom::try_from(v)?),
					Inner::Signed(v) => Ok(TryFrom::try_from(v)?),
					_ => TryFrom::try_from(u64::max_value()), // Always fails
				}
			}
		}
	}
}

macro_rules! impl_u {
	($t:ty) => {
		impl<'a> From<$t> for IntValue<'a> {
			fn from(value: $t) -> IntValue<'a> {
				IntValue{ inner: Inner::Unsigned(value as u64) }
			}
		}
		impl<'a> TryFrom<IntValue<'a>> for $t {
			type Error = TryFromIntError;
			fn try_from(value: IntValue<'a>) -> Result<$t, Self::Error> {
				match value.inner {
					Inner::Unsigned(v) => Ok(TryFrom::try_from(v)?),
					_ => TryFrom::try_from(-1), // Always fails
				}
			}
		}
	}
}

impl_s!(i8);
impl_s!(i16);
impl_s!(i32);
impl_s!(i64);
impl_u!(u8);
impl_u!(u16);
impl_u!(u32);
impl_u!(u64);

// TODO: Implement Ord and PartialOrd

/*
impl<'a> PartialEq for IntValue<'a> {
	fn eq(&self, other: &IntValue<'a>) -> bool {
		match self.inner {
			Inner::Unsigned(v) => Some(v) == other.get_u64(),
			Inner::Signed(v) => Some(v) == other.get_i64(),
			Inner::Big(ref d) => match other.inner {
				Inner::Big(ref d2) => d == d2,
				_ => false
			}
		}
	}
}

impl<'a> Eq for IntValue<'a> {}

impl<'a> fmt::Debug for IntValue<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self.inner {
			Inner::Unsigned(v) => write!(f, "{}", v),
			Inner::Signed(v) => write!(f, "{}", v),
			Inner::Big(ref d) => {
				if d.len() == 0 {
					write!(f, "0")
				} else {
					write!(f, "0x")?;
					for byte in &d[..] {
						write!(f, "{:02x}", byte)?;
					}
					Ok(())
				}
			}
		}
	}
}
*/

// TODO: update tests

#[test]
fn test_64() {
	assert_eq!(Int::from(5u64).get_i64(), Some(5));
	assert_eq!(Int::from(5i64).get_i64(), Some(5));
	assert_eq!(Int::from(5u16).get_i64(), Some(5));
	assert_eq!(Int::from(5i16).get_i64(), Some(5));
	assert_eq!(Int::from(5u64).get_u64(), Some(5));
	assert_eq!(Int::from(5i64).get_u64(), Some(5));
	assert_eq!(Int::from(-1).get_u64(), None);
	assert_eq!(Int::from(-1).get_i64(), Some(-1));
	assert_eq!(Int::from(u64::max_value()).get_i64(), None);
	assert_eq!(Int::from(u64::max_value()).get_u64(), Some(u64::max_value()));
	assert_eq!(Int::from(i64::max_value()).get_u64(), Some(i64::max_value() as u64));
}

#[test]
fn test_big() {
	assert_eq!(Int::Big([][..].into()).get_u64(), Some(0));
	assert_eq!(Int::Big([0, 0][..].into()).get_u64(), Some(0));
	assert_eq!(Int::Big([1, 0][..].into()).get_u64(), Some(256));
	assert_eq!(Int::Big([0xFF][..].into()).get_i64(), Some(-1));
	assert_eq!(Int::Big([0xFF, 0xFF][..].into()).get_i64(), Some(-1));
	assert_eq!(Int::Big([1, 0, 0, 0, 0, 0, 0, 0, 0][..].into()).get_u64(), None);
	assert_eq!(Int::Big([0xFF, 0, 0, 0, 0, 0, 0, 0][..].into()).get_u64(), None);
	assert_eq!(Int::Big([0, 0xFF, 0, 0, 0, 0, 0, 0, 0][..].into()).get_u64(), Some(0xFF000000_00000000));
	assert_eq!(Int::Big([0xFF, 0, 0, 0, 0, 0, 0, 0][..].into()).get_i64(), Some(-0x1000000_00000000));
}

#[test]
fn test_eq_64() {
	assert_eq!(Int::Unsigned(5), Int::Unsigned(5));
	assert_eq!(Int::Signed(5), Int::Signed(5));
	assert_eq!(Int::Signed(5), Int::Unsigned(5));
	assert_ne!(Int::Unsigned(5), Int::Unsigned(6));
	assert_ne!(Int::Signed(5), Int::Signed(6));
	assert_ne!(Int::Signed(5), Int::Unsigned(6));
}

#[test]
fn test_eq_big() {
	assert_eq!(Int::Big([5][..].into()), Int::Unsigned(5));
	assert_eq!(Int::Big([0xFF][..].into()), Int::Signed(-1));
	assert_eq!(
		Int::Big([0xFF][..].into()),
		Int::Big([0xFF][..].into()),
	);
	assert_eq!(
		Int::Big([][..].into()),
		Int::Big([0, 0][..].into()),
	);
	assert_eq!(
		Int::Big([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
		Int::Big([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
	);
	assert_eq!(
		Int::Big([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
		Int::Big([0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
	);
	assert_eq!(
		Int::Big([0xFF, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
		Int::Big([0xFF, 0xFF, 0xFF, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10][..].into()),
	);
}
