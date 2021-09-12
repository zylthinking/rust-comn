use crate::{callpos, cptr, mptr, nil, CallPos};
use std::{
    mem::transmute,
    ptr::addr_of_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

#[repr(C)]
#[derive(Debug)]
pub struct LkfNode(AtomicPtr<LkfNode>, AtomicPtr<CallPos>);

impl LkfNode {
    #[inline]
    pub fn new() -> LkfNode {
        LkfNode(nil!(), nil!())
    }

    #[inline]
    pub unsafe fn next(&mut self) -> *mut LkfNode {
        let ptr = self.0.load(Ordering::Relaxed);
        let next = (*ptr).0.load(Ordering::Relaxed);
        if ptr == nil!() || next == nil!() {
            return nil!();
        }

        if ptr != self {
            self.0 = (*ptr).0;
        }
        (*ptr).0 = nil!();
        (*ptr).1 = nil!();
        ptr
    }
}

impl Drop for LkfNode {
    fn drop(&mut self) {
        let pos = self.1.load(Ordering::Relaxed);
        if pos != nil!() {
            let pos: &CallPos = unsafe { &*pos };
            panic!("still linked: {}", pos);
        }
    }
}

pub struct Lkf {
    root: LkfNode,
    tail: AtomicPtr<*mut LkfNode>,
}
unsafe impl Sync for Lkf {}
unsafe impl Send for Lkf {}

impl Lkf {
    #[inline]
    pub fn new() -> Lkf {
        Lkf {
            root: LkfNode::new(),
            tail: nil!(),
        }
    }

    /// # Safety
    /// This fn is unsafe because it does not care about the lifetime of `node`.
    /// Caller must unlink `node` before its lifetime ends.
    #[inline]
    pub unsafe fn put(
        &self,
        node: *mut LkfNode,
        pos: *mut CallPos,
    ) -> Result<(), &'static CallPos> {
        let x = (*node)
            .1
            .compare_exchange(nil!(), pos, Ordering::Relaxed, Ordering::Relaxed);
        if let Err(pos) = x {
            return Err(&*pos);
        }

        let next = (*node).0.load(Ordering::Relaxed);
        let nextp = self.tail.swap(&mut next, Ordering::Relaxed);
        if nextp == nil!() {
            self.root.0.store(node, Ordering::Relaxed);
        } else {
            *nextp = node;
        }
        Ok(())
    }

    #[inline]
    pub fn get(&self) -> *mut LkfNode {
        let node = self.root.0.swap(nil!(), Ordering::Relaxed);
        if node == nil!() {
            return node;
        }

        let last = self.tail.swap(nil!(), Ordering::Relaxed);
        unsafe {
            *last = node;
        }
        mptr!(last)
    }

    /// # Safety
    /// This fn is unsafe because:
    ///
    /// It maybe load a have-been-freed memory block internally.
    /// The loaded data will not be used if it does have been freed.
    /// Caller must assure reading such a memory block will not cause any undefined behavior.
    pub unsafe fn get_one(&self) -> *mut LkfNode {
        let mut node = self.root.0.load(Ordering::Relaxed);
        while node != nil!() {
            // Dangerous: the node may have been freed.
            let ptr = addr_of_mut!((*node).0);
            // So the loading is technically undefined.
            // while, it is ok on most of platforms to do a simple loading
            // I won't use the loaded data if it is indeed freed
            let next = (*ptr).load(Ordering::Relaxed);
            // Rust promises AtomicPtr<T> has same layput as T
            let ptr = transmute(ptr);
            if next == nil!() {
                let r =
                    self.tail
                        .compare_exchange(ptr, nil!(), Ordering::Relaxed, Ordering::Relaxed);
                if r.is_ok() {
                    // There is only one entry in list,
                    // and the entry has been unlinked by the code up above.
                    (*node).1 = nil!();
                    break;
                }

                // There are at least 2 entries in list
                // no entry unlinked, fallthrough to unlink the 1st entry
                next = *r.err().unwrap();
            }

            let r = self
                .root
                .0
                .compare_exchange(node, next, Ordering::Relaxed, Ordering::Relaxed);
            if r.is_ok() {
                (*node).0 = nil!();
                (*node).1 = nil!();
                break;
            }
            node = r.err().unwrap();
        }
        node
    }
}

#[macro_export]
macro_rules! lkf_put_unsafe {
    ($list:expr, $node:expr) => {
        unsafe { ($list).put($node, callpos!()) }
    };
}

#[macro_export]
macro_rules! lkf_get {
    ($list:expr) => {
        ($list).get()
    };
}

#[macro_export]
macro_rules! lkf_next_unsafe {
    ($node:expr) => {
        unsafe { (*$node).next() }
    };
}
