#![allow(unused_imports)]
#![allow(non_camel_case_types)]

pub use pmacros::{_suffix, compile_time};
use std::alloc::{alloc, dealloc, Layout};

mod macros;
pub use macros::CallPos;

pub mod asyn;
pub mod atomic;
pub mod leadlock;

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
