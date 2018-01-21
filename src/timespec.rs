/// A point in time, relative to the unix epoch.
///
/// We have our own Timespec struct instead of using one from std or another
/// crate, because:
///
/// - Currently, the standard library doesn't provide any way to convert a
///   timestamp to a std::time::SystemTime without the possibility of a panic.
///
/// - The time crate does provide a proper Timespec struct, but is deprecated.
///
/// - Time chrono crate is a bit too big of a dependency.
///
/// - The types of the members of libc::timespec aren't the same on every platform.
///
/// The most used time types/libraries seem to provide a way to work with a
/// timestamp since the unix epoch with separate seconds (i64 or u64) and
/// nanoseconds (i32 or u32). For example:
///  `chrono::NativeDateTime::from_timestamp(secs, nsecs)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timespec {
	/// The number of seconds since unix epoch, possibly negative.
	pub sec: i64,
	/// The number of subsecond nanoseconds, 0 <= nsec < 1_000_000_000.
	pub nsec: u32,
}
