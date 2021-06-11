use crate::{
    atomic::{self, AtomicP},
    callpos, cptr, mptr, nil, CallPos,
};
use std::{
    future::Ready,
    mem::{size_of, transmute},
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
    usize,
};

#[repr(C)]
#[derive(Debug)]
pub struct LkfNode(*mut LkfNode, *const CallPos);

impl LkfNode {
    pub fn new() -> LkfNode {
        LkfNode(nil!(), nil!())
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

#[macro_export]
macro_rules! InitLkf {
    ($lkf:ident) => {
        let mut $lkf = $crate::Lkf::new();
        $lkf.init();
    };
}

#[macro_export]
macro_rules! lkf_put {
    ($list:expr, $node:expr) => {
        ($list).put($node, callpos!());
    };
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
    pub fn init(&mut self) {
        self.tail = unsafe { transmute(&self.root.0) };
    }

    #[inline]
    pub fn put(
        &mut self,
        node: *mut LkfNode,
        pos: &'static CallPos,
    ) -> Result<(), &'static CallPos> {
        unsafe {
            let x = (*node).1.atomic_compare_exchange(
                nil!(),
                pos,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            if let Err(pos) = x {
                return Err(&*pos);
            }

            let nextp: *mut *mut LkfNode = self.tail.atomic_swap(&mut (*node).0, Ordering::Relaxed);
            *nextp = node;
            Ok(())
        }
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

#[test]
fn lkf_test() {
    InitLkf!(q);

    let mut x = LkfNode::new();
    println!("{:?}, {:p}, {:p}", q.root, &x, q.tail);

    let _ = lkf_put!(&mut q, &mut x).unwrap();
    println!("{:?}", x);
    println!("{:?}, {:p}, {:p}", q.root, &x, q.tail);
    let _ = q.get();
    println!("{:?}, {:p}, {:p}", q.root, &x, q.tail);
    let _ = lkf_put!(q, &mut x).unwrap();
}
