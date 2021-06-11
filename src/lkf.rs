use crate::{callpos, cptr, mptr, nil, CallPos};
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
    tail: *mut *const LkfNode,
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
    pub fn put(&self, node: *const LkfNode, pos: &'static CallPos) -> Result<(), &'static CallPos> {
        unsafe {
            let ptr = AtomicPtr::new(mptr!((*node).1));
            println!("{:p}", (*node).1);

            if let Err(pos) =
                ptr.compare_exchange(nil!(), mptr!(pos), Ordering::Relaxed, Ordering::Relaxed)
            {
                return Err(&*pos);
            }
            ptr::write(mptr!(&(*node).1), pos);

            let ptr = AtomicPtr::new(self.tail);
            let nextp = ptr.swap(mptr!(&(*node).0), Ordering::Relaxed);

            println!("{:p}---------------", &(*node).0);
            *nextp = node;
            Ok(())
        }
    }

    #[inline]
    pub fn get(&self) -> *const LkfNode {
        let ptr = AtomicPtr::<LkfNode>::new(self.root.0);
        let node = ptr.swap(nil!(), Ordering::Relaxed);
        if node == nil!() {
            return node;
        }

        unsafe {
            println!(
                "----------------- {:?}, {:?}",
                *node,
                *(self.tail as *const LkfNode)
            );
        }

        let ptr = AtomicPtr::<*const LkfNode>::new(self.tail);
        let last = ptr.swap(mptr!(&self.root.0), Ordering::Relaxed);
        unsafe {
            *last = node;
        }
        cptr!(last)
    }
}

#[test]
fn lkf_test() {
    InitLkf!(q);

    let x = LkfNode::new();
    let _ = lkf_put!(&q, &x).unwrap();
    println!("{:?}", x);
    println!("{:?}, {:p}, {:p}", q.root, &x, q.tail);
    //let _ = q.get();
    //println!("{:?}", x);
    let _ = lkf_put!(q, &x).unwrap();
}
