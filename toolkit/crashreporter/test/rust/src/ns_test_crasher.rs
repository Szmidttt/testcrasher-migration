use std::fs;
use std::io::Write;
use windows_sys::Win32::System::{
    LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
    Memory::GetProcessHeap,
    Diagnostics::Debug::{
        SetUnhandledExceptionFilter, EXCEPTION_POINTERS,
    },
    Threading::{GetCurrentProcess, TerminateProcess}, 
    SystemInformation::GetSystemTimeAsFileTime
};
use windows_sys::Win32::Foundation::FILETIME;
use sadness_generator::SadnessFlavor;
#[cfg(feature = "moz_phc")]
use crate::bindings::root::{
    mozilla::phc,
    Rust_IsPHCAllocation,
    Rust_SetPHCState};
use crate::bindings::root::{
    Rust_MOZ_CRASH,
    Rust_moz_xmalloc,
    PureVirtualCall,
    ThrowException};
use std::ffi::c_void;
use std::mem::MaybeUninit;
#[cfg(feature = "moz_phc")]
use libc::{malloc, free};
#[cfg(target_os = "macos")]
use libc::{c_int, c_uint, c_char};

#[cfg(all(windows, target_arch = "x86_64"))]
extern "C" {
    fn x64CrashCFITest_NO_MANS_LAND(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_Launcher(returnpfn: u64, testProc: *mut c_void) -> u64;
    fn x64CrashCFITest_UnknownOpcode(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_PUSH_NONVOL(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_ALLOC_SMALL(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_ALLOC_LARGE(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_SAVE_NONVOL(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_SAVE_NONVOL_FAR(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_SAVE_XMM128(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_SAVE_XMM128_FAR(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_EPILOG(returnpfn: u64, ptr: *mut c_void) -> u64;
    fn x64CrashCFITest_EOF(returnpfn: u64, ptr: *mut c_void) -> u64;
}

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
#[cfg(feature = "moz_phc")]
const CRASH_PHC_USE_AFTER_FREE: i16 = 21;
#[cfg(feature = "moz_phc")]
const CRASH_PHC_DOUBLE_FREE: i16 = 22;
#[cfg(feature = "moz_phc")]
const CRASH_PHC_BOUNDS_VIOLATION: i16 = 23;
#[cfg(target_os = "windows")]
const CRASH_HEAP_CORRUPTION: i16 = 24;
#[cfg(target_os = "macos")]
const CRASH_EXC_GUARD: i16 = 25;
#[cfg(not(target_os = "windows"))]
const CRASH_STACK_OVERFLOW: i16 = 26;

#[cfg(target_os = "windows")]
unsafe fn heap_corruption() {
    let kernel32_name: Vec<u16> = "Kernel32.dll\0".encode_utf16().collect();
    let kernel32 = LoadLibraryW(kernel32_name.as_ptr());

    if kernel32 == 0 { return; }

    let func_name = b"HeapFree\0"; 
    let real_heap_free_addr = GetProcAddress(kernel32, func_name.as_ptr());

    if let Some(func_ptr) = real_heap_free_addr {
        let real_heap_free: unsafe extern "system" fn(isize, u32, *mut std::ffi::c_void) -> i32 = 
            std::mem::transmute(func_ptr);
        let heap = GetProcessHeap();
        let bad_pointer = 3 as *mut std::ffi::c_void;
        real_heap_free(heap, 0, bad_pointer);
    }
}

#[cfg(feature = "moz_phc")]
#[no_mangle]
pub extern "C" fn GetPHCAllocation(size: usize) -> *mut u8 {
    // A crude but effective way to get a PHC allocation.
    for _ in 0..2000000 {
        unsafe {
            let p = malloc(size) as *mut u8;
            if !p.is_null() && Rust_IsPHCAllocation(p as *mut c_void, std::ptr::null_mut()) {
                return p;
            }
            free(p as *mut c_void);
        }
    }
    panic!("failed to get a PHC allocation");
}

// This ensures tests have enough committed stack space.
// Must not be inlined, or the stack space would not be freed for the caller to use.
#[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
#[inline(never)]
unsafe fn reserve_stack() {
    const ELEMENTS: usize = (1024000 / std::mem::size_of::<FILETIME>()) + 1;
    let mut stackmem = MaybeUninit::<[FILETIME; ELEMENTS]>::uninit();
    let base_ptr = stackmem.as_mut_ptr() as *mut FILETIME;
    GetSystemTimeAsFileTime(base_ptr);
    GetSystemTimeAsFileTime(base_ptr.add(ELEMENTS - 1));
}

#[no_mangle]
pub unsafe extern "C" fn Crash(how: i16) {
    match how {
        CRASH_INVALID_POINTER_DEREF => {
            SadnessFlavor::Segfault.make_sad(); 
        },
        CRASH_PURE_VIRTUAL_CALL => {
            PureVirtualCall();
        },
        CRASH_OOM => {
            Rust_moz_xmalloc(usize::MAX);
            Rust_moz_xmalloc(usize::MAX);
            Rust_moz_xmalloc(usize::MAX);
        },
        CRASH_MOZ_CRASH => {
            Rust_MOZ_CRASH();
        },
        CRASH_ABORT => {
            SadnessFlavor::Abort.make_sad();
        },
        CRASH_UNCAUGHT_EXCEPTION => {
            ThrowException();
        },
        #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
        CRASH_X64CFI_UNKNOWN_OPCODE
        | CRASH_X64CFI_PUSH_NONVOL
        | CRASH_X64CFI_ALLOC_SMALL
        | CRASH_X64CFI_ALLOC_LARGE
        | CRASH_X64CFI_SAVE_NONVOL
        | CRASH_X64CFI_SAVE_NONVOL_FAR
        | CRASH_X64CFI_SAVE_XMM128
        | CRASH_X64CFI_SAVE_XMM128_FAR
        | CRASH_X64CFI_EPILOG => {
            unsafe {
                let fn_addr =resolve_cfi_func_addr(how);
                if fn_addr == 0 {
                    return;
                }
                let launcher_addr = resolve_cfi_func_addr(CRASH_X64CFI_LAUNCHER);
                if launcher_addr == 0 {
                    return;
                }
                reserve_stack();
                x64CrashCFITest_Launcher(0, fn_addr as *mut std::os::raw::c_void);
            }
        },
        #[cfg(feature = "moz_phc")]
        CRASH_PHC_USE_AFTER_FREE => {
            unsafe {
                let p = GetPHCAllocation(32);
                if !p.is_null() {
                    free(p as *mut c_void);
                    *p = 0;
                }
            }
        },
        #[cfg(feature = "moz_phc")]
        CRASH_PHC_DOUBLE_FREE => {
            unsafe {
                let p = GetPHCAllocation(64);
                if !p.is_null() {
                    free(p as *mut c_void);
                    free(p as *mut c_void);
                }
            }
        },
        #[cfg(feature = "moz_phc")]
        CRASH_PHC_BOUNDS_VIOLATION => {
            unsafe {
                let p = GetPHCAllocation(96);
                if !p.is_null() {
                    *p.offset(96) = 0;
                }
            }
        },
        #[cfg(target_os = "windows")]
        CRASH_HEAP_CORRUPTION => {
            heap_corruption();
            panic!("1");
        },
        #[cfg(target_os = "macos")]
        CRASH_EXC_GUARD => {
            exc_guard_crash();
        },
        #[cfg(not(target_os = "windows"))]
        CRASH_STACK_OVERFLOW => {
            SadnessFlavor::StackOverflow.make_sad(); 
        },
        _ => {panic!("1");}
    }
}

#[no_mangle]
pub extern "C" fn EnablePHC() {
    #[cfg(feature = "moz_phc")]
    unsafe {
        Rust_SetPHCState(phc::PHCState_Enabled);
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

#[cfg(all(windows, target_arch = "x86_64"))]
type CfiAsmFunc = unsafe extern "C" fn(u64, *mut c_void) -> u64;

#[cfg(all(windows, target_arch = "x86_64"))]
unsafe fn resolve_cfi_func_addr(fnid: i16) -> u64 {
    let target_func: CfiAsmFunc = match fnid {
        CRASH_X64CFI_NO_MANS_LAND => x64CrashCFITest_NO_MANS_LAND,
        CRASH_X64CFI_LAUNCHER => x64CrashCFITest_Launcher,
        CRASH_X64CFI_UNKNOWN_OPCODE => x64CrashCFITest_UnknownOpcode,
        CRASH_X64CFI_PUSH_NONVOL => x64CrashCFITest_PUSH_NONVOL,
        CRASH_X64CFI_ALLOC_SMALL => x64CrashCFITest_ALLOC_SMALL,
        CRASH_X64CFI_ALLOC_LARGE => x64CrashCFITest_ALLOC_LARGE,
        CRASH_X64CFI_SAVE_NONVOL => x64CrashCFITest_SAVE_NONVOL,
        CRASH_X64CFI_SAVE_NONVOL_FAR => x64CrashCFITest_SAVE_NONVOL_FAR,
        CRASH_X64CFI_SAVE_XMM128 => x64CrashCFITest_SAVE_XMM128,
        CRASH_X64CFI_SAVE_XMM128_FAR => x64CrashCFITest_SAVE_XMM128_FAR,
        CRASH_X64CFI_EPILOG => x64CrashCFITest_EPILOG,
        CRASH_X64CFI_EOF => x64CrashCFITest_EOF,
        _ => return 0,
    };
    // ret values point to jump table entries, not the actual function bodies.
    // Get the correct pointer by calling the function with returnpfn=1
    target_func(1, std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn GetWin64CFITestFnAddrOffset(fnid: i16) -> u32 {
    #[cfg(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu")))]
    // fnid uses the same constants as Crash().
    // Returns the RVA of the requested function.
    // Returns 0 on failure.
    unsafe{
        let fn_addr= resolve_cfi_func_addr(fnid);
        if fn_addr == 0 {
            return 0;
        }

        let dll_name: Vec<u16> = "testcrasher.dll\0".encode_utf16().collect();
        let module_base = GetModuleHandleW(dll_name.as_ptr()) as u64;

        if module_base == 0 {
            return 0;
        }

        (fn_addr - module_base) as u32
    }

    #[cfg(not(all(target_os = "windows", target_pointer_width = "64", target_arch = "x86_64", not(target_env = "gnu"))))]
    {
        let _ = fnid;
        0
    }
}


#[cfg(target_os = "macos")]
unsafe fn exc_guard_crash() {
    use std::ffi::CString;

    const GUARD_CLOSE: u32 = 1u32 << 0;
    const GUARD_DUP: u32 = 1u32 << 1;
    const GUARD_SOCKET_IPC: u32 = 1u32 << 2;
    const GUARD_FILEPORT: u32 = 1u32 << 3;
    const O_CREAT: i32 = 0x0200;
    const O_CLOEXEC: i32 = 0x1000000;
    const O_RDWR: i32 = 0x0002;

    type GuardedOpenNpT = unsafe extern "C" fn(
        *const i8,
        *const u64,
        u32,
        i32,
        u32,
    ) -> i32;

    let kernellib = libc::dlopen(
        b"/usr/lib/system/libsystem_kernel.dylib\0".as_ptr() as *const i8,
        libc::RTLD_GLOBAL,
    );

    if !kernellib.is_null() {
        let sym = libc::dlsym(kernellib, b"guarded_open_np\0".as_ptr() as *const i8);

        if !sym.is_null() {
            let guarded_open_np: GuardedOpenNpT = std::mem::transmute(sym);
            let guard: u64 = 0x123456789ABCDEF;

            let path = CString::new("/tmp/try.txt").unwrap();
            let fd = guarded_open_np(
                path.as_ptr(),
                &guard,
                GUARD_CLOSE | GUARD_DUP | GUARD_SOCKET_IPC | GUARD_FILEPORT,
                O_CREAT | O_CLOEXEC | O_RDWR,
                0o666,
            );

            if fd != -1 {
                libc::close(fd);
            }
        }
    }
}



