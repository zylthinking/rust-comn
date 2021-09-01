use crate::{
    atomic::{AtomicN, AtomicP},
    nil,
};
use std::{
    mem::forget,
    sync::{atomic::Ordering, Arc, Condvar, Mutex, MutexGuard},
};

pub struct cond<T> {
    cv: Condvar,
    uptr: Option<Arc<T>>,
}

impl<T> cond<T> {
    fn new() -> Self {
        cond {
            cv: Condvar::new(),
            uptr: None,
        }
    }

    fn wait(&self, mut g: MutexGuard<()>) -> Arc<T> {
        while self.uptr.is_none() {
            g = self.cv.wait(g).unwrap();
        }

        match self.uptr {
            Some(ref uptr) => uptr.clone(),
            None => unreachable!(),
        }
    }

    fn wake(&mut self, uptr: Arc<T>) {
        self.uptr = Some(uptr);
        self.cv.notify_one();
    }
}

pub struct LeadLock<T> {
    nr: i32,
    mux: Mutex<()>,
    mux_cond: Mutex<()>,
    cond: *const cond<T>,
    holder: *const cond<T>,
}

pub enum Locked<'a, T> {
    locked(MutexGuard<'a, ()>),
    unlocked(Arc<T>),
}

impl<T> LeadLock<T> {
    pub fn new() -> Self {
        let cond = Arc::new(cond::new());
        LeadLock {
            nr: 0,
            cond: Arc::into_raw(cond),
            holder: nil!(),
            mux: Mutex::new(()),
            mux_cond: Mutex::new(()),
        }
    }

    pub fn lock(&self) -> Locked<'_, T> {
        let n = self.nr.atomic_fetch_add(1, Ordering::Relaxed);
        if n > 1 {
            let g = self.mux_cond.lock().unwrap();
            let cond = unsafe {
                let cond = self.cond.atomic_load(Ordering::Relaxed);
                let cond = Arc::from_raw(cond);
                let c = cond.clone();
                forget(cond);
                c
            };
            return Locked::unlocked(cond.wait(g));
        }

        let g = self.mux.lock().unwrap();
        self.nr.atomic_store(0, Ordering::Relaxed);

        let cond = self
            .cond
            .atomic_swap(Arc::into_raw(Arc::new(cond::new())), Ordering::Relaxed);
        self.holder.atomic_store(cond, Ordering::Relaxed);
        Locked::locked(g)
    }

    pub fn unlock(&self, g: MutexGuard<'_, ()>, uptr: Arc<T>) {
        let holder = self.holder.atomic_swap(nil!(), Ordering::Relaxed);
        if holder == nil!() {
            return;
        }
        drop(g);
        // no new follower can reach holder
        drop(self.mux_cond.lock().unwrap());

        let mut cond = unsafe { Arc::from_raw(holder) };
        match Arc::get_mut(&mut cond) {
            Some(x) => x.wake(uptr),
            None => unreachable!(),
        }
    }
}
