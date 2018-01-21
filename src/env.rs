use fd;
use std;

use encoded_with_fds;
use values::EncodedArgdata;

extern "C" {
	fn program_get_raw_argdata(_: *mut *const u8, _: *mut usize);
}

/// Returns the argdata which this program was started with.
///
/// Only available on CloudABI targets.
pub fn argdata() -> EncodedArgdata<&'static [u8], fd::Identity> {
	unsafe {
		let mut data = std::ptr::null();
		let mut len = 0;
		program_get_raw_argdata(&mut data, &mut len);
		encoded_with_fds(std::slice::from_raw_parts(data, len), fd::Identity)
	}
}
