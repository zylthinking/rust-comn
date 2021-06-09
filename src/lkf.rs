use crate::{callpos, nil, CallPos};
use std::{
    future::Ready,
    mem::{size_of, transmute},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    usize,
};

pub struct LkfNode(*mut LkfNode, usize);

impl LkfNode {
    pub fn new() -> LkfNode {
        LkfNode(nil!(), 0)
    }
}

impl Drop for LkfNode {
    fn drop(&mut self) {
        if self.1 != 0 {
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
    pub fn put(&self, node: &mut LkfNode, pos: &'static CallPos) -> Result<(), &'static CallPos> {
        unsafe {
            let pos: usize = transmute(pos);
            let location: &AtomicUsize = transmute(&node.1);
            if let Err(pos) = location.compare_exchange(0, pos, Ordering::Relaxed, Ordering::Relaxed) {
                let pos: &'static CallPos = transmute(pos);
                return Err(pos);
            }

            let next_ptr: usize = transmute(&node.1);
            let tail_ptr: &AtomicUsize = transmute(&self.tail);
            let prev = tail_ptr.swap(next_ptr, Ordering::Relaxed);
            let prev: *mut _ = transmute(prev);
            *prev = node;
            Ok(())
        }
    }

    #[inline]
    pub fn get(&self) -> &'static usize {
        unsafe { &0 }
    }
}

// static inline struct lkf_node* lkf_node_get(struct lkf_list* list)
// {
//     struct lkf_node* ptr = __sync_lock_test_and_set(&(list->root.next), NULL);
//     if (ptr == NULL) {
//         return NULL;
//     }

//     struct lkf_node** last = __sync_lock_test_and_set(&(list->tail), &(list->root.next));
//     *last = ptr;
//     return (struct lkf_node *) last;
// }

#[test]
fn a() {
    InitLkf!(q);
    let mut x = LkfNode::new();
    let ok = q.put(&mut x, callpos!()).unwrap();
    let ok = q.put(&mut x, callpos!()).unwrap();

    // println!("{}", q.root)
}
