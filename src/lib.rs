#![allow(unused_imports)]

use std::alloc::{Layout, alloc, dealloc};
pub use pmacros::{_suffix, compile_time};

mod macros;
pub use macros::CallPos;

pub mod atomic;

mod my_handle;
pub use my_handle::MyHandle;

mod list_head;
pub use list_head::ListHead;

mod lkf;
pub use lkf::{Lkf, LkfNode};

pub unsafe fn malloc<T>() -> *mut T {
    alloc(Layout::new::<T>()) as *mut T
}

pub unsafe fn free<T>(ptr: *const T) {
    dealloc(ptr as *mut _, Layout::new::<T>());
}
