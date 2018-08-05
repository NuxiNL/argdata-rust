use fd;

use encoded_with_fds;
use values::EncodedArgdata;

/// Returns the argdata which this program was started with.
///
/// Only available on CloudABI targets.
pub fn argdata() -> EncodedArgdata<'static, fd::Identity> {
	argdata_impl()
}

#[cfg(target_os = "cloudabi")]
fn argdata_impl() -> EncodedArgdata<'static, fd::Identity> {
	use std;

	extern "C" {
		fn program_get_raw_argdata(_: *mut *const u8, _: *mut usize);
	}

	unsafe {
		let mut data = std::ptr::null();
		let mut len = 0;
		program_get_raw_argdata(&mut data, &mut len);
		encoded_with_fds(std::slice::from_raw_parts(data, len), fd::Identity)
	}
}

#[cfg(not(target_os = "cloudabi"))]
fn argdata_impl() -> EncodedArgdata<'static, fd::Identity> {
	encoded_with_fds(&[], fd::Identity)
}
