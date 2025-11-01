use std::ffi::CStr;
use minidump::*;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn DumpHasStream(dump_file: *const c_char, stream_type: u32) -> bool {
    let path_cstr = unsafe {CStr::from_ptr(dump_file)};
    let Ok(path_str) = path_cstr.to_str() else {
        return false;
    };
    let dump = match Minidump::read_path(path_str) {
        Ok(d)=> d,
        Err(_) =>return false,
    };
    match dump.get_raw_stream(stream_type){
        Ok(stream) => stream.len()>0,
        Err(_) => false,
    }
}
