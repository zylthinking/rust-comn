use std::{
    cell::UnsafeCell,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

pub struct MyHandle<T> {
    ptr: UnsafeCell<Option<Box<T>>>,
    stack: AtomicI32,
    detached: AtomicI32,
    freed: AtomicI32,
}

impl<T> MyHandle<T> {
    pub fn attach(ptr: Box<T>) -> Arc<MyHandle<T>> {
        Arc::new(MyHandle {
            stack: AtomicI32::new(1),
            detached: AtomicI32::new(0),
            freed: AtomicI32::new(0),
            ptr: UnsafeCell::new(Some(ptr)),
        })
    }

    fn get_with(&self, detach: i32, line: u32, file: &str) -> &Option<Box<T>> {
        let n = self.stack.fetch_add(1, Ordering::Release);
        let r = self
            .detached
            .compare_exchange(0, detach, Ordering::Relaxed, Ordering::Relaxed);
        if let Err(_) = r {
            self.put();
            return &None;
        };

        let optr = self.ptr.get();
        unsafe {
            assert!(
                n > 0 && (*optr).is_some(),
                "{} freed: {}, caller {}@{}",
                n,
                (*optr).is_none(),
                line,
                file
            );
            &*optr
        }
    }

    pub fn put(&self) {
        let n = self.stack.fetch_sub(1, Ordering::Release);
        assert!(n >= 1);
        if n > 1 {
            return;
        }

        let freed = self.freed.swap(1, Ordering::Relaxed);
        if freed == 1 {
            return;
        }

        let optr = self.ptr.get();
        unsafe {
            (*optr).take();
        }
    }

    pub fn get(&self) -> &Option<Box<T>> {
        self.get_with(0, line!(), file!())
    }

    pub fn dettach(&self) {
        if let &Some(_) = self.get_with(1, line!(), file!()) {
            self.stack.fetch_sub(1, Ordering::Relaxed);
            self.put();
        }
    }
}

impl<T> Drop for MyHandle<T> {
    fn drop(&mut self) {
        let detached = self.detached.load(Ordering::Relaxed);
        if detached == 0 {
            self.put();
        }
    }
}

unsafe impl<T> Sync for MyHandle<T> where T: Sync {}
