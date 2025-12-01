use std::fs;
use std::io::Write;
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleW,
    Diagnostics::Debug::{
        SetUnhandledExceptionFilter, EXCEPTION_POINTERS,
    },
    Threading::{GetCurrentProcess, TerminateProcess}, 
};
use sadness_generator::SadnessFlavor;
use crate::phc_bindings::root::mozilla::phc;

// Keep these in sync with CrashTestUtils.sys.mjs!
const CRASH_INVALID_POINTER_DEREF: i16 = 0;
const CRASH_PURE_VIRTUAL_CALL: i16 = 1;
const CRASH_OOM: i16 = 3;
const CRASH_MOZ_CRASH: i16 = 4;
const CRASH_ABORT: i16 = 5;
const CRASH_UNCAUGHT_EXCEPTION: i16 = 6;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_NO_MANS_LAND: i16 = 7;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_LAUNCHER: i16 = 8;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_UNKNOWN_OPCODE: i16 = 9;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_PUSH_NONVOL: i16 = 10;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_ALLOC_SMALL: i16 = 11;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_ALLOC_LARGE: i16 = 12;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_SAVE_NONVOL: i16 = 15;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_SAVE_NONVOL_FAR: i16 = 16;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_SAVE_XMM128: i16 = 17;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_SAVE_XMM128_FAR: i16 = 18;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_EPILOG: i16 = 19;
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
const CRASH_X64CFI_EOF: i16 = 20;
const CRASH_PHC_USE_AFTER_FREE: i16 = 21;
const CRASH_PHC_DOUBLE_FREE: i16 = 22;
const CRASH_PHC_BOUNDS_VIOLATION: i16 = 23;
#[cfg(target_os = "windows")]
const CRASH_HEAP_CORRUPTION: i16 = 24;
#[cfg(target_os = "macos")]
const CRASH_EXC_GUARD: i16 = 25;
#[cfg(not(target_os = "windows"))]
const CRASH_STACK_OVERFLOW: i16 = 26;


// // Helper functions for stack overflow (non-Windows)
// #[cfg(not(target_os = "windows"))]
// fn recurse(random: i64) -> i64 {
//     let mut buff: [u8; 256] = [0; 256];
//     let mut result = random;
    
//     let gibberish = b"This is gibberish";
//     let len = gibberish.len().min(buff.len());
//     buff[..len].copy_from_slice(&gibberish[..len]);
    
//     for c in &buff {
//         result = result.wrapping_add(*c as i64);
//     }
    
//     if result == 0 {
//         return result;
//     }
    
//     recurse(result).wrapping_add(1)
// }

// #[no_mangle]
// pub extern "C" fn Crash(how: i16) {
//     match how {
//         CRASH_INVALID_POINTER_DEREF => {
//             unsafe { SadnessFlavor::Segfault.make_sad(); }
//         },
//         #[cfg(not(target_os = "windows"))]
//         CRASH_STACK_OVERFLOW => {
//             unsafe { SadnessFlavor::StackOverflow.make_sad(); }
//         },
//         CRASH_ABORT => {
//             unsafe { SadnessFlavor::Abort.make_sad(); }
//         },
//         _ => {}
//     }
// }

#[no_mangle]
pub extern "C" fn EnablePHC() {
    #[cfg(feature = "moz_phc")]
    unsafe {
        phc::SetPHCState(phc::PHCState_Enabled);
    }
}

#[no_mangle]
pub extern "C" fn SaveAppMemory() -> u64 {
    let mut test_data = Box::new([0u8; 32]);
    for i in 0..32 {
        test_data[i] = i as u8;
    }
    let leaked_ref: &'static mut [u8; 32] = Box::leak(test_data);

    let addr = leaked_ref.as_ptr() as u64;

    if let Ok(mut file) = fs::File::create("crash-addr") {
        let _ = writeln!(file, "0x{:x}", addr);
    }

    addr
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn handle_exception(_exinfo: *const EXCEPTION_POINTERS) -> i32 {
    TerminateProcess(GetCurrentProcess(), 0);
    0
}

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "C" fn TryOverrideExceptionHandler() {
    unsafe {
        SetUnhandledExceptionFilter(Some(handle_exception));
    }
}

//TODO po migracji funkcji crash usunąc komentarz let fn_addr = ...
// #[no_mangle]
// pub extern "C" fn GetWin64CFITestFnAddrOffset(fnid: i16) -> u32 {
//     #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
//     // fnid uses the same constants as Crash().
//     // Returns the RVA of the requested function.
//     // Returns 0 on failure.
//     {
//         //let fn_addr = ... //TODO
//         let _ = fnid;
//         let fn_addr = 0;
//         if fn_addr == 0 {
//             return 0;
//         }

//         let dll_name: Vec<u16> = "testcrasher.dll\0".encode_utf16().collect();
//         let module_base = unsafe {GetModuleHandleW(dll_name.as_ptr()) as u64};

//         if module_base == 0 {
//             return 0;
//         }

//         (fn_addr - module_base) as u32
//     }

//     #[cfg(not(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu"))))]
//     {
//         let _ = fnid;
//         0
//     }
// }

// use std::ffi::CStr;
// use minidump::*;
// use std::os::raw::c_char;
// use std::fs;
// 

// #[cfg(any(feature = "moz_phc", not(target_os = "windows"), target_os = "macos"))]
// extern crate libc;


// // FFI declarations for external C++ functions
// #[cfg(feature = "moz_phc")]
// extern "C" {
//     fn PHC_IsPHCAllocation(ptr: *const u8, info: *mut u8) -> bool;
// }

// #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
// extern "C" {
//     fn x64CrashCFITest_Launcher(returnpfn: u64, test_proc: *mut std::os::raw::c_void) -> u64;
// }

// // This ensures tests have enough committed stack space.
// // Must not be inlined, or the stack space would not be freed for the caller to use.
// #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
// #[inline(never)]
// fn reserve_stack() {
//     unsafe {
//         // We must actually use the memory in some way that the compiler can't optimize away.
//         const ELEMENTS: usize = (1024000 / std::mem::size_of::<windows_impl::FILETIME>()) + 1;
//         let mut stackmem: [windows_impl::FILETIME; ELEMENTS] = std::mem::zeroed();
//         windows_impl::GetSystemTimeAsFileTime(&mut stackmem[0]);
//         windows_impl::GetSystemTimeAsFileTime(&mut stackmem[ELEMENTS - 1]);
//     }
// }

// // ThrowException - throws a C++ exception
// #[no_mangle]
// pub extern "C" fn ThrowException() {
//     // In Rust, we can't throw C++ exceptions directly, so we panic
//     // which will unwind similarly (though not identically to C++ exceptions)
//     panic!("Exception thrown");
// }

// // GetPHCAllocation - gets a PHC-allocated memory block
// #[cfg(feature = "moz_phc")]
// #[no_mangle]
// pub extern "C" fn GetPHCAllocation(size: usize) -> *mut u8 {
//     for _ in 0..2000000 {
//         unsafe {
//             let p = libc::malloc(size) as *mut u8;
//             if !p.is_null() && PHC_IsPHCAllocation(p, std::ptr::null_mut()) {
//                 return p;
//             }
//             libc::free(p as *mut libc::c_void);
//         }
//     }
//     panic!("failed to get a PHC allocation");
// }
// // Helper function for pure virtual call (Rust doesn't have this concept)
// fn pure_virtual_call() {
//     panic!("Pure virtual call simulation");
// }
// #[cfg(not(target_os = "windows"))]
// extern "C" fn overflow_stack_thread(arg: *mut std::os::raw::c_void) -> *mut std::os::raw::c_void {
//     unsafe {
//         let data = *(arg as *const i64);
//         let result = recurse(data);
//         result as *mut std::os::raw::c_void
//     }
// }

// #[no_mangle]
// pub extern "C" fn Crash(how: i16) {
//     match how {
//         CRASH_INVALID_POINTER_DEREF => {
//             unsafe {
//                 let foo: *mut i32 = 0x42 as *mut i32;
//                 *foo = 0;
//             }
//         }
//         CRASH_PURE_VIRTUAL_CALL => {
//             pure_virtual_call();
//         }
//         CRASH_OOM => {
//             // Attempt to allocate massive amounts of memory
//             let _ = vec![0u8; usize::MAX];
//             let _ = vec![0u8; usize::MAX];
//             let _ = vec![0u8; usize::MAX];
//         }
//         CRASH_MOZ_CRASH => {
//             panic!("MOZ_CRASH");
//         }
//         CRASH_ABORT => {
//             std::process::abort();
//         }
//         CRASH_UNCAUGHT_EXCEPTION => {
//             ThrowException();
//         }
//         #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
//         CRASH_X64CFI_UNKNOWN_OPCODE
//         | CRASH_X64CFI_PUSH_NONVOL
//         | CRASH_X64CFI_ALLOC_SMALL
//         | CRASH_X64CFI_ALLOC_LARGE
//         | CRASH_X64CFI_SAVE_NONVOL
//         | CRASH_X64CFI_SAVE_NONVOL_FAR
//         | CRASH_X64CFI_SAVE_XMM128
//         | CRASH_X64CFI_SAVE_XMM128_FAR
//         | CRASH_X64CFI_EPILOG => {
//             unsafe {
//                 let fn_addr = GetWin64CFITestMap_GetFunctionAddress(how);
//                 if fn_addr == 0 {
//                     return;
//                 }
//                 let launcher_addr = GetWin64CFITestMap_GetFunctionAddress(CRASH_X64CFI_LAUNCHER);
//                 if launcher_addr == 0 {
//                     return;
//                 }
//                 reserve_stack();
//                 x64CrashCFITest_Launcher(0, fn_addr as *mut std::os::raw::c_void);
//             }
//         }
//         #[cfg(feature = "moz_phc")]
//         CRASH_PHC_USE_AFTER_FREE => {
//             unsafe {
//                 let p = GetPHCAllocation(32);
//                 if !p.is_null() {
//                     libc::free(p as *mut libc::c_void);
//                     *p = 0;
//                 }
//             }
//         }
//         #[cfg(feature = "moz_phc")]
//         CRASH_PHC_DOUBLE_FREE => {
//             unsafe {
//                 let p = GetPHCAllocation(64);
//                 if !p.is_null() {
//                     libc::free(p as *mut libc::c_void);
//                     libc::free(p as *mut libc::c_void);
//                 }
//             }
//         }
//         #[cfg(feature = "moz_phc")]
//         CRASH_PHC_BOUNDS_VIOLATION => {
//             unsafe {
//                 let p = GetPHCAllocation(96);
//                 if !p.is_null() {
//                     *p.offset(96) = 0;
//                 }
//             }
//         }
//         #[cfg(target_os = "windows")]
//         CRASH_HEAP_CORRUPTION => {
//             unsafe {
//                 heap_corruption_crash();
//             }
//         }
//         #[cfg(target_os = "macos")]
//         CRASH_EXC_GUARD => {
//             unsafe {
//                 exc_guard_crash();
//             }
//         }
//         #[cfg(not(target_os = "windows"))]
//         CRASH_STACK_OVERFLOW => {
//             unsafe {
//                 let mut thread_id: libc::pthread_t = std::mem::zeroed();
//                 let data: i64 = 1337;
//                 let rv = libc::pthread_create(
//                     &mut thread_id,
//                     std::ptr::null(),
//                     overflow_stack_thread,
//                     &data as *const i64 as *mut std::os::raw::c_void,
//                 );
//                 if rv == 0 {
//                     libc::pthread_join(thread_id, std::ptr::null_mut());
//                 }
//             }
//         }
//         _ => {}
//     }
// }

// // Platform-specific crash implementations
// #[cfg(target_os = "windows")]
// unsafe fn heap_corruption_crash() {
//     use std::ffi::OsStr;
//     use std::os::windows::ffi::OsStrExt;

//     let kernel32_name: Vec<u16> = OsStr::new("Kernel32.dll")
//         .encode_wide()
//         .chain(std::iter::once(0))
//         .collect();
//     let kernel32 = windows_impl::LoadLibraryW(kernel32_name.as_ptr());

//     if kernel32 != std::ptr::null_mut() {
//         let heap_free_name = b"HeapFree\0";
//         let heap_free_ptr = windows_impl::GetProcAddress(kernel32, heap_free_name.as_ptr() as *const i8);

//         if heap_free_ptr != std::ptr::null_mut() {
//             let heap = windows_impl::GetProcessHeap();
//             let bad_pointer = 3 as *mut std::os::raw::c_void;

//             type HeapFreeT = unsafe extern "system" fn(isize, u32, *mut std::os::raw::c_void) -> i32;
//             let heap_free: HeapFreeT = std::mem::transmute(heap_free_ptr);
//             heap_free(heap as isize, 0, bad_pointer);
//         }
//     }
// }

// #[cfg(target_os = "macos")]
// unsafe fn exc_guard_crash() {
//     use std::ffi::CString;

//     const GUARD_CLOSE: u32 = 1u32 << 0;
//     const GUARD_DUP: u32 = 1u32 << 1;
//     const GUARD_SOCKET_IPC: u32 = 1u32 << 2;
//     const GUARD_FILEPORT: u32 = 1u32 << 3;
//     const O_CREAT: i32 = 0x0200;
//     const O_CLOEXEC: i32 = 0x1000000;
//     const O_RDWR: i32 = 0x0002;

//     type GuardedOpenNpT = unsafe extern "C" fn(
//         *const i8,
//         *const u64,
//         u32,
//         i32,
//         u32,
//     ) -> i32;

//     let kernellib = libc::dlopen(
//         b"/usr/lib/system/libsystem_kernel.dylib\0".as_ptr() as *const i8,
//         libc::RTLD_GLOBAL,
//     );

//     if !kernellib.is_null() {
//         let sym = libc::dlsym(kernellib, b"guarded_open_np\0".as_ptr() as *const i8);

//         if !sym.is_null() {
//             let guarded_open_np: GuardedOpenNpT = std::mem::transmute(sym);
//             let guard: u64 = 0x123456789ABCDEF;

//             let path = CString::new("/tmp/try.txt").unwrap();
//             let fd = guarded_open_np(
//                 path.as_ptr(),
//                 &guard,
//                 GUARD_CLOSE | GUARD_DUP | GUARD_SOCKET_IPC | GUARD_FILEPORT,
//                 O_CREAT | O_CLOEXEC | O_RDWR,
//                 0o666,
//             );

//             if fd != -1 {
//                 libc::close(fd);
//             }
//         }
//     }
// }


// static mut TEST_DATA: [u8; 32] = [0; 32];


// #[cfg(feature = "moz_phc")]
// extern "C" {
//     fn PHC_SetState(state: i32);
// }

// #[no_mangle]
// pub extern "C" fn EnablePHC() {
//     #[cfg(feature = "moz_phc")]
//     unsafe {
//         PHC_SetState(1);
//     }
// }

// #[cfg(target_os = "windows")]
// mod windows_impl {
//     use std::os::raw::c_void;

//     #[repr(C)]
//     pub struct EXCEPTION_POINTERS {
//         _unused: [u8; 0],
//     }

//     pub type LONG = i32;
//     pub type HANDLE = *mut c_void;

//     #[cfg(all(target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
//     #[repr(C)]
//     pub struct FILETIME {
//         pub dw_low_date_time: u32,
//         pub dw_high_date_time: u32,
//     }

//     extern "system" {
//         pub fn TerminateProcess(process: HANDLE, exit_code: u32) -> i32;
//         pub fn GetCurrentProcess() -> HANDLE;
//         pub fn SetUnhandledExceptionFilter(
//             handler: Option<unsafe extern "system" fn(*mut EXCEPTION_POINTERS) -> LONG>,
//         ) -> Option<unsafe extern "system" fn(*mut EXCEPTION_POINTERS) -> LONG>;
//         pub fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
//         pub fn LoadLibraryW(filename: *const u16) -> *mut c_void;
//         pub fn GetProcAddress(module: *mut c_void, proc_name: *const i8) -> *mut c_void;
//         pub fn GetProcessHeap() -> isize;
//         #[cfg(all(target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
//         pub fn GetSystemTimeAsFileTime(system_time: *mut FILETIME);
//     }
// }


// #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
// extern "C" {
//     fn GetWin64CFITestMap_GetFunctionAddress(fnid: i16) -> u64;
// }

// #[no_mangle]
// pub extern "C" fn GetWin64CFITestFnAddrOffset(fnid: i16) -> u32 {
//     #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
//     unsafe {
//         let fn_addr = GetWin64CFITestMap_GetFunctionAddress(fnid);
//         if fn_addr == 0 {
//             return 0;
//         }

//         let dll_name: Vec<u16> = "testcrasher.dll\0".encode_utf16().collect();
//         let module_base = windows_impl::GetModuleHandleW(dll_name.as_ptr()) as u64;

//         if module_base == 0 {
//             return 0;
//         }

//         (fn_addr - module_base) as u32
//     }

//     #[cfg(not(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu"))))]
//     {
//         let _ = fnid;
//         0
//     }
// }
