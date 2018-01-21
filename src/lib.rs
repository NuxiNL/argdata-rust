#![feature(i128_type)]
#![feature(try_from)]

extern crate byteorder;

use std::convert::TryFrom;
use std::fmt;
use std::io;
use std::ops::Deref;

#[cfg(target_os="cloudabi")]
pub mod env;

// TODO: Decide what should be public, and what should be in the root namespace.
pub mod fd;
mod integer;
mod subfield;
mod timespec;
mod values;
pub mod container;

pub use integer::Integer;
pub use timespec::Timespec;
pub use values::*;

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
	Fd(fd::EncodedFd<'a>),
	Float(f64),
	Int(Integer<'a>),
	Str(&'a str),
	Timestamp(Timespec),
	Map(&'a (Map + 'a)),
	Seq(&'a (Seq + 'a)),
}

impl<'a> Value<'a> {
	fn get_type(&self) -> Type {
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

/// An error while reading argdata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

	/// The data represents a timestamp that does not fit in a Timespec.
	TimestampOutOfRange,

	/// The data contains a subfield (of a map or seq) with an incomplete or too large length.
	InvalidSubfield,

	/// The data contains a map with an incomplete key-value pair.
	InvalidKeyValuePair,

	/// The data represents a file descriptor that doesn't exist.
	/// (Possibly because there were no file descriptors attached at all.)
	InvalidFdNumber(u32),
}

/// The reason why a read_*() call didn't return a value, when there was no read error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoFit {

	/// The value is too high or low to fit in the requested type.
	OutOfRange,

	/// The value seems to be of a different type.
	DifferentType,
}

/// The reason why an Argdata::read_*() call didn't return a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotRead {

	/// The value couldn't be read, because it wouldn't fit in the requested type.
	/// (Because it the value is of a different type, or isn't big enough.)
	NoFit(NoFit),

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

impl From<NoFit> for NotRead {
	fn from(e: NoFit) -> NotRead {
		NotRead::NoFit(e)
	}
}

pub trait Argdata {

	fn read<'a>(&'a self) -> Result<Value<'a>, ReadError> {
		let t = self.get_type()?;
		let result = (|| match t {
			Type::Null      => Ok(Value::Null),
			Type::Binary    => Ok(Value::Binary(self.read_binary()?)),
			Type::Bool      => Ok(Value::Bool(self.read_bool()?)),
			Type::Fd        => Ok(Value::Fd(self.read_encoded_fd()?)),
			Type::Float     => Ok(Value::Float(self.read_float()?)),
			Type::Int       => Ok(Value::Int(self.read_int_value()?)),
			Type::Str       => Ok(Value::Str(self.read_str()?)),
			Type::Timestamp => Ok(Value::Timestamp(self.read_timestamp()?)),
			Type::Map       => Ok(Value::Map(self.read_map()?)),
			Type::Seq       => Ok(Value::Seq(self.read_seq()?)),
		})();
		match result {
			Ok(v) => Ok(v),
			Err(NotRead::Error(e)) => Err(e),
			Err(NotRead::NoFit(_)) => panic!("get_type() and read_<type>() are inconsistent"),
		}
	}

	fn get_type(&self) -> Result<Type, ReadError> {
		Ok(self.read()?.get_type())
	}

	fn read_null(&self) -> Result<(), NotRead> {
		match self.read()? {
			Value::Null => Ok(()),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_binary<'a>(&'a self) -> Result<&'a [u8], NotRead> {
		match self.read()? {
			Value::Binary(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_bool(&self) -> Result<bool, NotRead> {
		match self.read()? {
			Value::Bool(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_encoded_fd(&self) -> Result<fd::EncodedFd, NotRead> {
		match self.read()? {
			Value::Fd(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_float(&self) -> Result<f64, NotRead> {
		match self.read()? {
			Value::Float(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_int_value<'a>(&'a self) -> Result<Integer<'a>, NotRead> {
		match self.read()? {
			Value::Int(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_map<'a>(&'a self) -> Result<&'a (Map + 'a), NotRead> {
		match self.read()? {
			Value::Map(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_seq<'a>(&'a self) -> Result<&'a (Seq + 'a), NotRead> {
		match self.read()? {
			Value::Seq(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_str<'a>(&'a self) -> Result<&'a str, NotRead> {
		match self.read()? {
			Value::Str(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn read_timestamp(&self) -> Result<Timespec, NotRead> {
		match self.read()? {
			Value::Timestamp(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	fn serialized_length(&self) -> usize;

	fn serialize(&self, writer: &mut io::Write) -> io::Result<()>;
}

pub trait ArgdataExt {
	fn read_int<'a, T: TryFrom<Integer<'a>>>(&'a self) -> Result<T, NotRead>;
	fn read_fd(&self) -> Result<fd::Fd, NotRead>;
}

impl<A> ArgdataExt for A where A: Argdata + ?Sized {
	fn read_int<'a, T: TryFrom<Integer<'a>>>(&'a self) -> Result<T, NotRead> {
		self.read_int_value().and_then(|v|
			TryFrom::try_from(v).map_err(|_| NoFit::OutOfRange.into())
		)
	}

	fn read_fd(&self) -> Result<fd::Fd, NotRead> {
		self.read_encoded_fd().and_then(|fd|
			fd.fd().map_err(|_| ReadError::InvalidFdNumber(fd.raw_encoded_fd_number()).into())
		)
	}
}

pub enum ArgdataValue<'a> {
	Encoded(EncodedArgdata<&'a [u8], &'a (fd::ConvertFd + 'a)>),
	Reference(&'a (Argdata + 'a)),
}

impl<'a> Deref for ArgdataValue<'a> {
	type Target = Argdata + 'a;
	fn deref(&self) -> &Self::Target {
		match self {
			&ArgdataValue::Encoded(ref argdata) => argdata,
			&ArgdataValue::Reference(argdata) => argdata,
		}
	}
}

pub trait Map {
	fn iter_map_next<'a>(&'a self, cookie: &mut usize) ->
		Option<Result<(ArgdataValue<'a>, ArgdataValue<'a>), ReadError>>;
}

pub trait Seq {
	fn iter_seq_next<'a>(&'a self, cookie: &mut usize) ->
		Option<Result<ArgdataValue<'a>, ReadError>>;
}

impl<'a> Map + 'a {
	pub fn iter_map(&'a self) -> MapIterator<'a> {
		MapIterator{
			map: self,
			cookie: 0
		}
	}
}

impl<'a> Seq + 'a {
	pub fn iter_seq(&'a self) -> SeqIterator<'a> {
		SeqIterator{
			seq: self,
			cookie: 0
		}
	}
}

pub struct MapIterator<'a> {
	map: &'a (Map + 'a),
	cookie: usize,
}

pub struct SeqIterator<'a> {
	seq: &'a (Seq + 'a),
	cookie: usize,
}

impl<'a> Iterator for MapIterator<'a> {
	type Item = Result<(ArgdataValue<'a>, ArgdataValue<'a>), ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.map.iter_map_next(&mut self.cookie)
	}
}

impl<'a> Iterator for SeqIterator<'a> {
	type Item = Result<ArgdataValue<'a>, ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		self.seq.iter_seq_next(&mut self.cookie)
	}
}

struct FmtError<T>(Result<T, ReadError>);

impl<T: fmt::Debug> fmt::Debug for FmtError<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self.0 {
			Ok(ref value) => value.fmt(f),
			Err(ref err) => write!(f, "error(\"{:?}\")", err),
		}
	}
}

impl<'a> fmt::Debug for ArgdataValue<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		self.deref().fmt(f)
	}
}

impl<'a> fmt::Debug for Argdata + 'a {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		FmtError(self.read()).fmt(f)
	}
}

impl<'a> fmt::Debug for Value<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match self {
			&Value::Null => write!(f, "null"),
			&Value::Binary(val) => write!(f, "binary({:?})", val),
			&Value::Bool(val) => write!(f, "{}", val),
			&Value::Fd(fd) => write!(f, "fd({})", fd.raw_encoded_fd_number()),
			&Value::Float(val) => write!(f, "{}", val), // TODO: pick formatter that keeps all precision
			&Value::Int(ref val) => write!(f, "{:?}", val),
			&Value::Str(val) => write!(f, "{:?}", val),
			&Value::Timestamp(ref val) => write!(f, "timestamp({}, {})", val.sec, val.nsec),
			&Value::Map(val) => {
				let it = val.iter_map().map(|x| match x {
					Ok((k, v)) => (FmtError(Ok(k)), FmtError(Ok(v))),
					Err(e) => (FmtError(Err(e)), FmtError(Err(e))),
				});
				f.debug_map().entries(it).finish()
			}
			&Value::Seq(val) => {
				let it = val.iter_seq().map(|x| FmtError(x));
				f.debug_list().entries(it).finish()
			}
		}
	}
}

trait ToArgdata {
	type Type: Argdata;
	fn to_argdata(self) -> Self::Type;
}

#[test]
fn debug_fmt() {
	let argdata = EncodedArgdata(b"\
		\x06\x87\x08Hello\x00\x87\x08World\x00\x81\x02\x82\x02\x01\x86\x09\
		\x70\xF1\x80\x29\x15\x84\x05\x58\xe5\xd9\x80\x83\x06\x80\x80\
	");

	assert_eq!(
		format!("{:?}", &argdata as &Argdata),
		"{\"Hello\": \"World\", false: true, timestamp(485, 88045333): 5826009, null: {null: null}}"
	);

	let argdata = EncodedArgdata(b"\
		\x07\x81\x02\x82\x02\x01\x80\x87\x08Hello\x00\x81\x06\x81\x07\
	");

	assert_eq!(
		format!("{:?}", &argdata as &Argdata),
		"[false, true, null, \"Hello\", {}, []]"
	);
}

// TODO:
// Make member(s) of all types::* private, add argdata::from() constructor for all.
// Fd/Resource (template arg?)
// Owned stuff (encoded, seq, map, binary, str, bigint, ..?)
// Fix/update/make Tests

/*
fn test() {
	argdata::from(());
	argdata::from(b"asdf");
	argdata::from(true);
	argdata::from(1.23);
	argdata::from(17);
	argdata::from("asdf");
	argdata::from(argdata::Timespec{ sec: 1, nsec: 2 });
	argdata::from(&[(&val1, &val2), (&val3, &val4)]);
	argdata::from(&[&val1, &val2, &val3]);
}
*/
