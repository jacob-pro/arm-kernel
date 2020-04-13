use alloc::alloc::{GlobalAlloc, Layout};
use crate::bindings::{malloc, calloc, free, realloc};
use cty::{c_uint, c_void};

struct NewLibAlloc;

unsafe impl GlobalAlloc for NewLibAlloc {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size() as c_uint) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        calloc(layout.size() as c_uint, 1) as *mut u8
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        realloc(ptr as *mut c_void, new_size as c_uint) as *mut u8
    }
}

#[global_allocator]
static A: NewLibAlloc = NewLibAlloc;

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error(_: Layout) -> ! {
    panic!("Allocation error")
}
