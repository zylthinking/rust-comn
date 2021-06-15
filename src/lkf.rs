use crate::{atomic::AtomicP, callpos, cptr, mptr, nil, CallPos};
use std::{mem::transmute, sync::atomic::Ordering};

#[repr(C)]
#[derive(Debug)]
pub struct LkfNode(*mut LkfNode, *const CallPos);

impl LkfNode {
    #[inline]
    pub fn new() -> LkfNode {
        LkfNode(nil!(), nil!())
    }

    #[inline]
    pub unsafe fn next(&mut self) -> *mut LkfNode {
        let ptr = self.0;
        if ptr == nil!() || (*ptr).0 == nil!() {
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
        if self.1 != nil!() {
            let pos: &CallPos = unsafe { transmute(self.1) };
            panic!("still linked: {}", pos);
        }
    }
}

pub struct Lkf {
    root: LkfNode,
    tail: *mut *mut LkfNode,
}

impl Lkf {
    #[inline]
    pub fn new() -> Lkf {
        Lkf {
            root: LkfNode::new(),
            tail: nil!(),
        }
    }

    #[inline]
    pub unsafe fn put(
        &mut self,
        node: *mut LkfNode,
        pos: &'static CallPos,
    ) -> Result<(), &'static CallPos> {
        let x =
            (*node)
                .1
                .atomic_compare_exchange(nil!(), pos, Ordering::Relaxed, Ordering::Relaxed);
        if let Err(pos) = x {
            return Err(&*pos);
        }

        let nextp: *mut *mut LkfNode = self.tail.atomic_swap(&mut (*node).0, Ordering::Relaxed);
        if nextp == nil!() {
            self.root.0.atomic_store(node, Ordering::Relaxed);
        } else {
            *nextp = node;
        }
        Ok(())
    }

    #[inline]
    pub fn get(&mut self) -> *mut LkfNode {
        let node = self.root.0.atomic_swap(nil!(), Ordering::Relaxed);
        if node == nil!() {
            return node;
        }

        let last = self.tail.atomic_swap(&mut self.root.0, Ordering::Relaxed);
        unsafe {
            *last = node;
        }
        mptr!(last)
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
