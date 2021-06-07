use crate::nil;
use std::{
    mem::{size_of, transmute},
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

pub struct LkfNode(*mut LkfNode, bool);

impl LkfNode {
    pub fn new() -> LkfNode {
        LkfNode(nil!(), false)
    }
}

impl Drop for LkfNode {
    fn drop(&mut self) {
        if self.1 {
            panic!("still linked")
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
    pub fn put(&self, node: &mut LkfNode) {
        node.1 = true;

        unsafe {
            let next_ptr: usize = transmute(&node.0);
            let tail_ptr: &AtomicUsize = transmute(&self.tail);
            let prev = tail_ptr.swap(next_ptr, Ordering::Relaxed);
            let prev: *mut _ = transmute(prev);
            *prev = node;
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

    println!("{}", q.root)
}
