use alloc::{boxed::Box, vec, vec::Vec};
pub const MAX_INPUT: usize = 4*1024*1024;
pub const MAX_OUTPUT: usize = 1024*1024;
pub fn allocate(length: i32) -> i32 {
    if length<=0 || length as usize>MAX_INPUT { return 0; }
    Box::into_raw(vec![0u8;length as usize].into_boxed_slice()) as *mut u8 as usize as i32
}
/// Safety: only allocations returned by allocate or call, freed exactly once.
pub unsafe fn free(pointer: i32, length: i32) {
    if pointer!=0 && length>0 {
        unsafe { drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(pointer as u32 as usize as *mut u8,length as usize))); }
    }
}
/// Safety: host provides a live, bounded input allocation, distinct from output.
pub unsafe fn call<F:FnOnce(&[u8])->Vec<u8>>(pointer: i32,length: i32,handler:F) -> i64 {
    if pointer==0 || length<=0 || length as usize>MAX_INPUT { return 0; }
    #[cfg(target_arch="wasm32")]
    if pointer as u32 as u64 + length as u64 > (core::arch::wasm32::memory_size(0)*65536) as u64 { return 0; }
    let input=unsafe { core::slice::from_raw_parts(pointer as u32 as usize as *const u8,length as usize) };
    let output=handler(input); if output.is_empty() || output.len()>MAX_OUTPUT { return 0; }
    let length=output.len() as u64;
    let pointer=Box::into_raw(output.into_boxed_slice()) as *mut u8 as usize as u32;
    (((pointer as u64)<<32)|length) as i64
}
#[cfg(target_arch="wasm32")]
#[link(wasm_import_module="arex_v1")]
unsafe extern "C" { fn diagnostic(pointer:i32,length:i32)->i32; }
pub fn diagnostic_text(text:&str)->Result<(),crate::bounded::Error> {
    crate::bounded::text(text,256)?;
    #[cfg(target_arch="wasm32")]
    unsafe { crate::bounded::ensure(diagnostic(text.as_ptr() as i32,text.len() as i32)==0)?; }
    Ok(())
}
#[macro_export]
macro_rules! export_v1 {
    ($plan:path,$parse:path,$nav_plan:path,$nav_parse:path) => {
        #[unsafe(no_mangle)] pub extern "C" fn arex_alloc(n:i32)->i32 { $crate::abi::allocate(n) }
        #[unsafe(no_mangle)] pub extern "C" fn arex_free(p:i32,n:i32) { unsafe { $crate::abi::free(p,n) } }
        #[unsafe(no_mangle)] pub extern "C" fn plan_requests(p:i32,n:i32)->i64 { unsafe { $crate::abi::call(p,n,$plan) } }
        #[unsafe(no_mangle)] pub extern "C" fn parse_responses(p:i32,n:i32)->i64 { unsafe { $crate::abi::call(p,n,$parse) } }
        #[unsafe(no_mangle)] pub extern "C" fn plan_navigation(p:i32,n:i32)->i64 { unsafe { $crate::abi::call(p,n,$nav_plan) } }
        #[unsafe(no_mangle)] pub extern "C" fn parse_navigation(p:i32,n:i32)->i64 { unsafe { $crate::abi::call(p,n,$nav_parse) } }
    }
}
