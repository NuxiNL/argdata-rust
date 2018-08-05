use IntValue;
use Map;
use Seq;
use StrValue;
use Timespec;
use fd;

/// The type of an argdata value.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Type {
	Null,
	Binary,
	Bool,
	Fd,
	Float,
	Int,
	Str,
	Timestamp,
	Map,
	Seq,
}

/// A (borrowed) argdata value.
pub enum Value<'a, 'd: 'a> {
	Null,
	Binary(&'d [u8]),
	Bool(bool),
	Fd(fd::EncodedFd<&'a fd::ConvertFd>),
	Float(f64),
	Int(IntValue<'d>),
	Str(StrValue<'d>),
	Timestamp(Timespec),
	Map(&'a Map<'d>),
	Seq(&'a Seq<'d>),
}

impl<'a, 'd: 'a> Value<'a, 'd> {
	pub fn get_type(&self) -> Type {
		match self {
			&Value::Null         => Type::Null,
			&Value::Binary(_)    => Type::Binary,
			&Value::Bool(_)      => Type::Bool,
			&Value::Fd(_)        => Type::Fd,
			&Value::Float(_)     => Type::Float,
			&Value::Int(_)       => Type::Int,
			&Value::Str(_)       => Type::Str,
			&Value::Timestamp(_) => Type::Timestamp,
			&Value::Map(_)       => Type::Map,
			&Value::Seq(_)       => Type::Seq,
		}
	}
}