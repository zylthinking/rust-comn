use crate::{
    atomic::{AtomicN, AtomicP},
    nil,
};
use std::{
    cell::UnsafeCell,
    mem::forget,
    ops::Fn,
    sync::{atomic::Ordering, Arc, Condvar, Mutex, MutexGuard},
};

struct Cond<T> {
    cv: Condvar,
    uptr: UnsafeCell<Option<Arc<T>>>,
}

impl<T> Cond<T> {
    fn new() -> Self {
        Cond {
            cv: Condvar::new(),
            uptr: UnsafeCell::new(None),
        }
    }

    fn wait(&self, mut g: MutexGuard<()>) -> Arc<T> {
        let uptr = unsafe { &mut *self.uptr.get() };
        while uptr.is_none() {
            g = self.cv.wait(g).unwrap();
        }

        match uptr {
            Some(ref uptr) => uptr.clone(),
            None => unreachable!(),
        }
    }

    fn wake(&self, any: Arc<T>) {
        let uptr = unsafe { &mut *self.uptr.get() };
        *uptr = Some(any);
        self.cv.notify_all();
    }
}

pub enum R<'a, T> {
    G(MutexGuard<'a, ()>),
    V(Arc<T>),
}

pub struct LeadLock<T> {
    nr: i32,
    mux: Mutex<()>,
    mux_cond: Mutex<()>,
    cond: *const Cond<T>,
    holder: *const Cond<T>,
}

impl<T> Default for LeadLock<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Sync for LeadLock<T> {}
unsafe impl<T> Send for LeadLock<T> {}

impl<T> LeadLock<T> {
    pub fn new() -> Self {
        let cond = Arc::new(Cond::new());
        LeadLock {
            nr: 0,
            cond: Arc::into_raw(cond),
            holder: nil!(),
            mux: Mutex::new(()),
            mux_cond: Mutex::new(()),
        }
    }

    pub fn lock(&self) -> R<'_, T> {
        let n = self.nr.atomic_fetch_add(1, Ordering::Relaxed);
        if n > 0 {
            let g = self.mux_cond.lock().unwrap();
            let cond = unsafe {
                let cond = self.cond.atomic_load(Ordering::Relaxed);
                let cond = Arc::from_raw(cond);
                let c = cond.clone();
                forget(cond);
                c
            };
            return R::V(cond.wait(g));
        }

        let g = self.mux.lock().unwrap();
        self.nr.atomic_store(0, Ordering::Relaxed);

        let cond = self
            .cond
            .atomic_swap(Arc::into_raw(Arc::new(Cond::new())), Ordering::Relaxed);
        self.holder.atomic_store(cond, Ordering::Relaxed);
        R::G(g)
    }

    /// # Safety
    /// the function is unsafe because g must be the one returned from lock()
    pub unsafe fn unlock(&self, g: MutexGuard<'_, ()>, uptr: Arc<T>) {
        let holder = self.holder.atomic_swap(nil!(), Ordering::Relaxed);
        if holder == nil!() {
            return;
        }
        drop(g);
        // no new follower can reach holder
        drop(self.mux_cond.lock().unwrap());
        Arc::from_raw(holder).wake(uptr);
    }

    pub fn single_flight<F: FnOnce() -> Arc<T>>(&self, f: F) -> Arc<T> {
        match self.lock() {
            R::G(g) => {
                let any = f();
                unsafe { self.unlock(g, any.clone()) };
                any
            }
            R::V(any) => any,
        }
    }
}
