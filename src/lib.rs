#![feature(i128_type)]
#![feature(try_from)]

extern crate byteorder;

use std::convert::TryFrom;
use std::io;

/// Access to the program environment.
///
/// Only available on CloudABI.
#[cfg(target_os="cloudabi")]
pub mod env;

/// Traits used for `Seq` and `Map` value implementations.
pub mod container_traits;

/// All the things related to file descriptors.
pub mod fd;

mod debug;
mod errors;
mod integer;
mod map;
mod reference;
mod seq;
mod subfield;
mod timespec;

pub use errors::{ReadError, NoFit, NotRead};
pub use integer::Integer;
pub use map::{Map, MapIterator};
pub use reference::ArgdataRef;
pub use seq::{Seq, SeqIterator};
pub use timespec::Timespec;

#[path="values/mod.rs"]
mod values_;

pub use values_::{
	encoded,
	encoded_with_fds,
	null,
	binary,
	bool,
	float,
	process_fd,
	encoded_fd,
	invalid_fd,
	bigint,
	int,
	map,
	seq,
	str,
	timestamp
};

/// Implementations of specific `Argdata` types.
/// Use the functions in the root of this crate to create them.
pub mod values {
	pub use values_::{
		EncodedArgdata,
		Null,
		Binary,
		Bool,
		Float,
		BigInt,
		Int,
		Map,
		Seq,
		Str,
		Timestamp,
	};
}

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
	Int(Integer<'d>),
	Str(&'d str),
	Timestamp(Timespec),
	Map(&'a Map<'d>),
	Seq(&'a Seq<'d>),
}

impl<'a, 'd: 'a> Value<'a, 'd> {
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

/// An argdata value.
///
/// Note for implementers of this trait: Although all read methods have provided implementations,
/// they are implemented in terms of eachother. You either need to provide:
///
///  - the `read()` method, or
///  - the `get_type()` method and implementations of *all* `read_*()` methods.
///
/// Do the latter if `read()` would do anything non-trivial, to keep things efficient.
///
/// `get_type()` and `read_*()` need to be consistent, which means that `read_$TYPE()` for the type
/// returned by `get_type()` may *not* return an `Err(NotRead::NoFit)`. Otherwise, `read()` will
/// panic.
pub trait Argdata<'d> {

	/// Read the value.
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError> where 'd: 'a {
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

	/// Read the type of the value.
	fn get_type(&self) -> Result<Type, ReadError> {
		Ok(self.read()?.get_type())
	}

	/// Check if the value is null.
	fn read_null(&self) -> Result<(), NotRead> {
		match self.read()? {
			Value::Null => Ok(()),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a binary blob, and read it if it is.
	fn read_binary(&self) -> Result<&'d [u8], NotRead> {
		match self.read()? {
			Value::Binary(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a boolean, and read it if it is.
	fn read_bool(&self) -> Result<bool, NotRead> {
		match self.read()? {
			Value::Bool(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a file descriptor, and return it if it is.
	///
	/// Even though this function succeeds (returns an `Ok()`), converting the returned `EncodedFd`
	/// to an `Fd` might still fail.
	///
	/// Note: You probably want to use [`read_fd`](trait.ArgdataExt.html#tymethod.read_fd) instead.
	fn read_encoded_fd<'a>(&'a self) -> Result<fd::EncodedFd<&'a fd::ConvertFd>, NotRead> where 'd: 'a {
		match self.read()? {
			Value::Fd(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a float, and read it if it is.
	fn read_float(&self) -> Result<f64, NotRead> {
		match self.read()? {
			Value::Float(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is an integer, and read it if it is.
	///
	/// Note: You might want to use [`read_int`](trait.ArgdataExt.html#tymethod.read_int) instead to
	/// directly get a primitive type like `i32` or `u64`.
	fn read_int_value(&self) -> Result<Integer<'d>, NotRead> {
		match self.read()? {
			Value::Int(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a map, and get access to it if it is.
	fn read_map<'a>(&'a self) -> Result<&'a Map<'d>, NotRead> where 'd: 'a {
		match self.read()? {
			Value::Map(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a seq, and get access to it if it is.
	fn read_seq<'a>(&'a self) -> Result<&'a Seq<'d>, NotRead> where 'd: 'a {
		match self.read()? {
			Value::Seq(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a string, and read it if it is.
	fn read_str(&self) -> Result<&'d str, NotRead> {
		match self.read()? {
			Value::Str(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a timestamp, and read it if it is.
	fn read_timestamp(&self) -> Result<Timespec, NotRead> {
		match self.read()? {
			Value::Timestamp(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Serialize the argdata to the given writer.
	///
	/// Exactly `self.serialized_bytes()` bytes are written to the writer, if no error occurs.
	///
	/// File descriptors are mapped using `fd_map`.
	/// If it is None, encoded file descriptors will be kept as is, and actual
	/// file descriptors will be encoded as `-1` (invalid).
	fn serialize(&self, writer: &mut io::Write, fd_map: Option<&mut fd::FdMapping>) -> io::Result<()>;

	/// The number of bytes that `self.serialize()` will write.
	fn serialized_length(&self) -> usize;
}

/// Extra methods for `Argdata` values.
pub trait ArgdataExt<'d> {
	/// Read an integer, and convert it to the requested type if it fits.
	fn read_int<T: TryFrom<Integer<'d>>>(&self) -> Result<T, NotRead>;

	/// Read a file descriptor and convert it to an `Fd`.
	fn read_fd(&self) -> Result<fd::Fd, NotRead>;
}

impl<'d, A> ArgdataExt<'d> for A where A: Argdata<'d> + ?Sized {
	fn read_int<T: TryFrom<Integer<'d>>>(&self) -> Result<T, NotRead> {
		self.read_int_value().and_then(|v|
			TryFrom::try_from(v).map_err(|_| NoFit::OutOfRange.into())
		)
	}

	fn read_fd(&self) -> Result<fd::Fd, NotRead> {
		self.read_encoded_fd().and_then(|fd|
			fd.fd().map_err(|_| ReadError::InvalidFdNumber(fd.raw_encoded_number()).into())
		)
	}
}

// TODO:
// owning datastructures
// Fix/update/make Tests

#[allow(dead_code)]
fn example<'d>(ad: &Argdata<'d>) {
	// If this stops compiling, then something is wrong
	// with the lifetimes of Argdata. :)

	let mut sock_fd = None;
	let mut read_fd = None;
	let mut message = None;

	let mut it = ad.read_map().expect("argdata should be a map").iter_map();
	while let Some(Ok((key, val))) = it.next() {
		match key.read_str().expect("keys should be strings") {
			"socket"  => sock_fd = val.read_fd().ok(),
			"logfile" => read_fd = val.read_fd().ok(),
			"message" => message = val.read_str().ok(),
			_ => {}
		}
	}

	drop(sock_fd);
	drop(read_fd);
	drop(message);
}
