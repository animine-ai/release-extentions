    use core::alloc::{GlobalAlloc,Layout};
    use core::sync::atomic::{AtomicUsize,Ordering};
    struct Arena;
    #[repr(align(16))] struct Heap([u8;16*1024*1024]);
    static mut HEAP:Heap=Heap([0;16*1024*1024]);
    static USED:AtomicUsize=AtomicUsize::new(0);
    unsafe impl GlobalAlloc for Arena {
        unsafe fn alloc(&self,l:Layout)->*mut u8 {
            let mut old=USED.load(Ordering::Relaxed);
            let base=core::ptr::addr_of_mut!(HEAP.0).cast::<u8>() as usize;
            loop {let Some(address)=base.checked_add(old).and_then(|v|v.checked_add(l.align()-1)) else{core::arch::wasm32::unreachable();};let aligned=(address&!(l.align()-1))-base;let Some(end)=aligned.checked_add(l.size()) else{core::arch::wasm32::unreachable();};if end>16*1024*1024{core::arch::wasm32::unreachable();}
                match USED.compare_exchange_weak(old,end,Ordering::Relaxed,Ordering::Relaxed){Ok(_)=>return unsafe{core::ptr::addr_of_mut!(HEAP.0).cast::<u8>().add(aligned)},Err(v)=>old=v}
            }
        }
        unsafe fn dealloc(&self,_p:*mut u8,_l:Layout) {}
    }
    // Each host operation uses a fresh instance; the bounded arena dies with it.
    #[global_allocator] static ALLOC:Arena=Arena;
    #[panic_handler] fn panic(_: &core::panic::PanicInfo)->!{core::arch::wasm32::unreachable()}
    arex_sdk::export_v1!(super::plan,super::parse,super::nav_plan,super::nav_parse);
