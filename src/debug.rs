use crate::{Argdata, ArgdataRef, ReadError, Value};
use std::fmt;
use std::ops::Deref;

struct FmtError<T>(Result<T, ReadError>);

impl<T: fmt::Debug> fmt::Debug for FmtError<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self.0 {
			Ok(ref value) => value.fmt(f),
			Err(ref err) => write!(f, "error(\"{:?}\")", err),
		}
	}
}

impl<'a, 'd> fmt::Debug for ArgdataRef<'a, 'd> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.deref().fmt(f)
	}
}

impl<'a, 'd> fmt::Debug for dyn Argdata<'d> + 'a {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		FmtError(self.read()).fmt(f)
	}
}

impl<'a, 'd> fmt::Debug for Value<'a, 'd> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self {
			Value::Null => write!(f, "null"),
			Value::Binary(val) => write!(f, "binary({:?})", val),
			Value::Bool(val) => val.fmt(f),
			Value::Fd(fd) => write!(f, "fd({})", fd.raw_encoded_number()),
			Value::Float(val) => fmt::Debug::fmt(val, f),
			Value::Int(val) => fmt::Debug::fmt(val, f),
			Value::Str(val) => fmt::Debug::fmt(&FmtError(val.as_str().map_err(Into::into)), f),
			Value::Timestamp(val) => write!(f, "timestamp({}, {})", val.sec, val.nsec),
			Value::Map(val) => {
				let it = val.map(|x| match x {
					Ok((k, v)) => (FmtError(Ok(k)), FmtError(Ok(v))),
					Err(e) => (FmtError(Err(e)), FmtError(Err(e))),
				});
				f.debug_map().entries(it).finish()
			}
			Value::Seq(val) => {
				let it = val.map(FmtError);
				f.debug_list().entries(it).finish()
			}
		}
	}
}

#[test]
fn debug_fmt() {
	let argdata = crate::encoded(
		&b"\x06\x87\x08Hello\x00\x87\x08World\x00\x81\x02\x82\x02\x01\x86\x09\
		\x70\xF1\x80\x29\x15\x84\x05\x58\xe5\xd9\x80\x83\x06\x80\x80"[..],
	);

	assert_eq!(
		format!("{:?}", &argdata as &dyn Argdata),
		"{\"Hello\": \"World\", false: true, timestamp(485, 88045333): 5826009, null: {null: null}}"
	);

	let argdata =
		crate::encoded(&b"\x07\x81\x02\x82\x02\x01\x80\x87\x08Hello\x00\x81\x06\x81\x07"[..]);

	assert_eq!(
		format!("{:?}", &argdata as &dyn Argdata),
		"[false, true, null, \"Hello\", {}, []]"
	);
}
