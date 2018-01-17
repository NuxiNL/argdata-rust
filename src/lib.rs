#![feature(try_from)]

extern crate byteorder;

use std::convert::TryFrom;
use std::time::SystemTime;

mod intvalue;
pub use intvalue::IntValue;

mod types;
pub use types::*;

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

pub enum Value<'a> {
	Null,
	Binary(&'a [u8]),
	Bool(bool),
	Fd(u32), // TODO
	Float(f64),
	Int(IntValue<'a>),
	Str(&'a str),
	Timestamp(SystemTime),
	//Map(Map<'a>),
	//Seq(Seq<'a>),
}

impl<'a> Value<'a> {
	fn get_type(&self) -> Type {
		match self {
			&Value::Null         => Type::Null,
			&Value::Binary(_)    => Type::Binary,
			&Value::Bool(_)      => Type::Bool,
			&Value::Fd(_)        => Type::Fd, // TODO
			&Value::Float(_)     => Type::Float,
			&Value::Int(_)       => Type::Int,
			&Value::Str(_)       => Type::Str,
			&Value::Timestamp(_) => Type::Timestamp,
			// TODO &Value::Map(_)       => Type::Map,
			// TODO &Value::Seq(_)       => Type::Seq,
		}
	}
}

/// An error while reading argdata.
#[derive(Debug)]
pub enum ReadError {

	/// The data contained the given tag, which doesn't correspond to any known type.
	InvalidTag(u8),

	/// The data represents a string, but it wasn't null-terminated.
	MissingNullTerminator,

	/// The data represents a string, but it contained invalid UTF-8.
	InvalidUtf8,

	/// The data represents a boolean, but it contained a value other than 'false' or 'true'.
	InvalidBoolValue,

	/// The data represents a float, but wasn't exactly 64 bits.
	InvalidFloatLength,

	/// The data represents a file descriptor, but wasn't exactly 32 bits.
	InvalidFdLength,

	/// The data represents a timestamp out of the range of SystemTime.
	TimestampOutOfRange,
}

/// The reason why an Argdata::read_*() call didn't return a value.
#[derive(Debug)]
pub enum NotRead {

	/// The value couldn't be read, because it seems to be of another type.
	///
	/// Another read_*() call probably works.
	OtherType,

	/// The value seems to be of the requested type, but it couldn't be read
	/// because of an error.
	///
	/// No other read_*() call will work.
	Error(ReadError),
}

impl From<ReadError> for NotRead {
	fn from(e: ReadError) -> NotRead {
		NotRead::Error(e)
	}
}

pub trait Argdata<'a> {

	fn read(&'a self) -> Result<Value<'a>, ReadError> {
		let t = self.get_type()?;
		let result = (|| match t {
			Type::Null      => Ok(Value::Null),
			Type::Binary    => Ok(Value::Binary(self.read_binary()?)),
			Type::Bool      => Ok(Value::Bool(self.read_bool()?)),
			Type::Fd        => unimplemented!(), //TODO self.read_fd()?,
			Type::Float     => Ok(Value::Float(self.read_float()?)),
			Type::Int       => Ok(Value::Int(self.read_int_value()?)),
			Type::Str       => Ok(Value::Str(self.read_str()?)),
			Type::Timestamp => Ok(Value::Timestamp(self.read_timestamp()?)),
			Type::Map       => unimplemented!(), //TODO self.read_map()?,
			Type::Seq       => unimplemented!(), //TODO self.read_seq()?,
		})();
		match result {
			Ok(v) => Ok(v),
			Err(NotRead::Error(e)) => Err(e),
			Err(NotRead::OtherType) => panic!("get_type() and read_<type>() are inconsistent"),
		}
	}

	fn get_type(&'a self) -> Result<Type, ReadError> {
		Ok(self.read()?.get_type())
	}

	fn read_null(&'a self) -> Result<(), NotRead> {
		match self.read()? {
			Value::Null => Ok(()),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_binary(&'a self) -> Result<&'a [u8], NotRead> {
		match self.read()? {
			Value::Binary(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_bool(&'a self) -> Result<bool, NotRead> {
		match self.read()? {
			Value::Bool(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	//fn read_raw_fd(&self) -> Result<u32, NotRead>;
	//fn read_fd(&'a self) -> Result<TODO, NotRead>;

	fn read_float(&'a self) -> Result<f64, NotRead> {
		match self.read()? {
			Value::Float(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_int_value(&'a self) -> Result<IntValue<'a>, NotRead> {
		match self.read()? {
			Value::Int(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_str(&'a self) -> Result<&'a str, NotRead> {
		match self.read()? {
			Value::Str(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	fn read_timestamp(&'a self) -> Result<SystemTime, NotRead> {
		match self.read()? {
			Value::Timestamp(v) => Ok(v),
			_ => Err(NotRead::OtherType),
		}
	}

	//fn read_map(&self) -> ...;
	//fn read_seq(&self) -> ...;

	fn serialized_length(&self) -> usize;
	fn max_number_of_fds(&self) -> usize { 0 }
	fn serialize_into(&self, buf: &mut [u8]); // TODO: fds: Option<&mut &mut [u32]>);

	fn serialize_append_to_vec(&self, buf: &mut Vec<u8>) {
		let len = self.serialized_length();
		buf.reserve(len);
		unsafe {
			let buflen = buf.len();
			self.serialize_into(std::slice::from_raw_parts_mut(buf[buflen..].as_mut_ptr(), len));
			buf.set_len(buflen + len);
		}
	}

	fn serialize_to_vec(&self) -> Vec<u8> {
		let mut buf = Vec::new();
		self.serialize_append_to_vec(&mut buf);
		buf
	}
}

impl<'a> Argdata<'a> {

	pub fn read_int<T: TryFrom<IntValue<'a>>>(&'a self) -> Result<T, NotRead> {
		self.read_int_value().and_then(|v|
			TryFrom::try_from(v).map_err(|_| NotRead::OtherType)
		)
	}

}

fn _bla(_x: &Argdata<'static>) {}

// TODO:
// OwnedEncoded(&'a [u8])
// Fd TODO
// Int(Int<'a>)
// Timestamp(Int<'a>)
// Seq(&'a [&Argdata<'a>])
// OwnedSeq(Vec<Box<Argdata<'a>>>)
// Map
// OwnedMap
// .. ?
