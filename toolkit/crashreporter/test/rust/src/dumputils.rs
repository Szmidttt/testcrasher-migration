use std::ffi::CStr;
use minidump::*;
use std::os::raw::c_char;
use std::fs;

// Return true if the specified minidump contains a stream of |stream_type|.
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

// Return true if the specified minidump contains a memory region
// that contains the instruction pointer from the exception record.
#[no_mangle]
pub extern "C" fn DumpHasInstructionPointerMemory(dump_file: *const c_char) -> bool {
    let path_cstr = unsafe { CStr::from_ptr(dump_file) };
    let Ok(path_str) = path_cstr.to_str() else {
        return false;
    };
    let dump = match Minidump::read_path(path_str) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let exception = match dump.get_stream::<MinidumpException>() {
        Ok(e) => e,
        Err(_) => return false,
    };

    let memory_list = match dump.get_stream::<MinidumpMemoryList>() {
        Ok(m) => m,
        Err(_) => return false,
    };

    let system_info = match dump.get_stream::<MinidumpSystemInfo>() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let context = match exception.context(&system_info, None) {
        Some(ctx) => ctx,
        None => return false,
    };

    let instruction_pointer = context.get_instruction_pointer();

    memory_list.memory_at_address(instruction_pointer).is_some()
}

// This function tests for a very specific condition. It finds
// an address in a file, "crash-addr", in the CWD. It checks
// that the minidump has a memory region starting at that
// address. The region must be 32 bytes long and contain the
// values 0 to 31 as bytes, in ascending order.
#[no_mangle]
pub extern "C" fn DumpCheckMemory(dump_file: *const c_char) -> bool {
    let path_cstr = unsafe { CStr::from_ptr(dump_file) };
    let Ok(path_str) = path_cstr.to_str() else {
        return false;
    };
    let dump = match Minidump::read_path(path_str) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let memory_list = match dump.get_stream::<MinidumpMemoryList>() {
        Ok(m) => m,
        Err(_) => return false,
    };

    let addr_str = match fs::read_to_string("crash-addr") {
        Ok(s) => s,
        Err(_) => return false,
    };

    let addr_str = addr_str.trim();
    let addr: u64 = if addr_str.starts_with("0x") || addr_str.starts_with("0X") {
        match u64::from_str_radix(&addr_str[2..], 16) {
            Ok(a) => a,
            Err(_) => return false,
        }
    } else {
        match u64::from_str_radix(addr_str, 16) {
            Ok(a) => a,
            Err(_) => return false,
        }
    };

    let _ = fs::remove_file("crash-addr");

    let region = match memory_list.memory_at_address(addr) {
        Some(r) => r,
        None => return false,
    };

    if region.bytes.len() != 32 {
        return false;
    }

    for i in 0..32 {
        if region.bytes[i] != i as u8 {
            return false;
        }
    }

    true
}