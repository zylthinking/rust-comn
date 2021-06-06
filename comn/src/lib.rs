#![allow(unused_imports)]

use std::alloc::alloc;
use std::alloc::dealloc;
use std::alloc::Layout;

mod macros;
pub use pmacros::compile_time as compile_time;

mod my_handle;
pub use my_handle::MyHandle;

mod list_head;
pub use list_head::ListHead;

pub unsafe fn malloc<T>() -> *mut T {
    alloc(Layout::new::<T>()) as *mut T
}

pub unsafe fn free<T>(ptr: *const T) {
    dealloc(ptr as *mut _, Layout::new::<T>());
}
