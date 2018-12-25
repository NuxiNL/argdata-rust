/// Replacement for std::convert::TryFrom for stable Rust.
pub trait TryFrom<T>: Sized {
	type Error;
	fn try_from(value: T) -> Result<Self, Self::Error>;
}

macro_rules! min_value {
	(u64 => $to:ty) => { 0 };
	(i64 => $to:ty) => { <$to>::min_value() };
}

macro_rules! impl_try_from {
	($from:tt => $to:ty) => {
		impl TryFrom<$from> for $to {
			type Error = ();
			fn try_from(value: $from) -> Result<$to, ()> {
				if
					value >= min_value!($from => $to) as $from &&
					value <= <$to>::max_value() as $from
				{
					Ok(value as $to)
				} else {
					Err(())
				}
			}
		}
	};
}

impl_try_from!(u64 => u8);
impl_try_from!(u64 => u16);
impl_try_from!(u64 => u32);
impl_try_from!(u64 => u64);
impl_try_from!(u64 => i8);
impl_try_from!(u64 => i16);
impl_try_from!(u64 => i32);
impl_try_from!(u64 => i64);

impl_try_from!(i64 => i8);
impl_try_from!(i64 => i16);
impl_try_from!(i64 => i32);
impl_try_from!(i64 => i64);
