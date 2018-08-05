#![feature(try_from)]

//! **Please note:**
//! This crate is not yet stable.
//! The interface can and will change.
//! Almost nothing is tested yet.
//! Deserialization probably works, but serialization has bugs.

extern crate byteorder;

use std::convert::TryFrom;
use std::io;

/// Access to the program environment.
pub mod env;

/// Traits used for `Seq` and `Map` value implementations.
pub mod container_traits;

/// All the things related to file descriptors.
pub mod fd;

mod debug;
mod errors;
mod intvalue;
mod map;
mod reference;
mod seq;
mod strvalue;
mod subfield;
mod timespec;
mod value;

pub use errors::{NoFit, NotRead, ReadError};
pub use intvalue::IntValue;
pub use map::{Map, MapIterator};
pub use reference::ArgdataRef;
pub use seq::{Seq, SeqIterator};
pub use strvalue::StrValue;
pub use timespec::Timespec;
pub use value::{Type, Value};

#[path = "values/mod.rs"]
mod values_;

pub use values_::{
	bigint, binary, bool, encoded, encoded_fd, encoded_with_fds, float, int, invalid_fd, map, null,
	process_fd, seq, str, timestamp,
};

/// Implementations of specific `Argdata` types.
/// Use the functions in the root of this crate to create them.
pub mod values {
	pub use values_::{
		BigInt, Binary, Bool, EncodedArgdata, Float, Int, Map, Null, Seq, Str, Timestamp,
	};
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
	fn read<'a>(&'a self) -> Result<Value<'a, 'd>, ReadError>
	where
		'd: 'a,
	{
		let t = self.get_type()?;
		let result = match t {
			Type::Null      => Ok(Value::Null),
			Type::Binary    => self.read_binary()    .map(Value::Binary),
			Type::Bool      => self.read_bool()      .map(Value::Bool),
			Type::Fd        => self.read_encoded_fd().map(Value::Fd),
			Type::Float     => self.read_float()     .map(Value::Float),
			Type::Int       => self.read_int_value() .map(Value::Int),
			Type::Str       => self.read_str_value() .map(Value::Str),
			Type::Timestamp => self.read_timestamp() .map(Value::Timestamp),
			Type::Map       => self.read_map()       .map(Value::Map),
			Type::Seq       => self.read_seq()       .map(Value::Seq),
		};
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
	fn read_encoded_fd<'a>(&'a self) -> Result<fd::EncodedFd<&'a fd::ConvertFd>, NotRead>
	where
		'd: 'a,
	{
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
	fn read_int_value(&self) -> Result<IntValue<'d>, NotRead> {
		match self.read()? {
			Value::Int(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a map, and get access to it if it is.
	fn read_map<'a>(&'a self) -> Result<&'a Map<'d>, NotRead>
	where
		'd: 'a,
	{
		match self.read()? {
			Value::Map(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a seq, and get access to it if it is.
	fn read_seq<'a>(&'a self) -> Result<&'a Seq<'d>, NotRead>
	where
		'd: 'a,
	{
		match self.read()? {
			Value::Seq(v) => Ok(v),
			_ => Err(NoFit::DifferentType.into()),
		}
	}

	/// Check if the value is a string, and read it if it is.
	///
	/// Note: You probably want to use [`read_str`](trait.ArgdataExt.html#tymethod.read_str) instead
	/// to directly get a `&str`.
	fn read_str_value(&self) -> Result<StrValue<'d>, NotRead> {
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
	fn serialize(
		&self,
		writer: &mut io::Write,
		fd_map: Option<&mut fd::FdMapping>,
	) -> io::Result<()>;

	/// The number of bytes that `self.serialize()` will write.
	fn serialized_length(&self) -> usize;
}

/// Extra methods for `Argdata` values.
pub trait ArgdataExt<'d> {
	/// Read an integer, and convert it to the requested type if it fits.
	fn read_int<T: TryFrom<IntValue<'d>>>(&self) -> Result<T, NotRead>;

	/// Read a file descriptor and convert it to an `Fd`.
	fn read_fd(&self) -> Result<fd::Fd, NotRead>;

	/// Read a string, and check if it's valid UTF-8.
	fn read_str(&self) -> Result<&'d str, NotRead>;
}

impl<'d, A> ArgdataExt<'d> for A
where
	A: Argdata<'d> + ?Sized,
{
	fn read_int<T: TryFrom<IntValue<'d>>>(&self) -> Result<T, NotRead> {
		TryFrom::try_from(self.read_int_value()?).map_err(|_| NoFit::OutOfRange.into())
	}

	fn read_fd(&self) -> Result<fd::Fd, NotRead> {
		self.read_encoded_fd()?
			.to_fd()
			.map_err(|raw| ReadError::InvalidFdNumber(raw).into())
	}

	fn read_str(&self) -> Result<&'d str, NotRead> {
		Ok(self.read_str_value()?.as_str()?)
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
			"socket" => sock_fd = val.read_fd().ok(),
			"logfile" => read_fd = val.read_fd().ok(),
			"message" => message = val.read_str().ok(),
			_ => {}
		}
	}

	drop(sock_fd);
	drop(read_fd);
	drop(message);
}
