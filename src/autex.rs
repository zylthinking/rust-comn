use crate::{atomic::AtomicN, list_entry, ListHead};
use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::Ordering, Arc, Mutex, MutexGuard},
    task::{Context, Poll, Wake, Waker},
};

pub struct Guard<'a>(&'a Autex);
impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        self.0.unlock();
    }
}

pub struct Autex {
    hold: i32,
    mux: Mutex<Box<ListHead>>,
}
unsafe impl Sync for Autex {}
unsafe impl Send for Autex {}

#[allow(non_camel_case_types)]
struct autex<'a> {
    autx: &'a Autex,
    ent: ListHead,
    waker: Option<Waker>,
}

impl<'a> autex<'a> {
    fn new(aux: &'a Autex) -> Self {
        autex {
            autx: aux,
            ent: ListHead::new(),
            waker: None,
        }
    }
}

impl<'a> Future for &mut autex<'a> {
    type Output = Guard<'a>;
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let r = self
            .autx
            .hold
            .atomic_compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed);
        if r.is_ok() {
            return Poll::Ready(Guard(self.autx));
        }

        self.waker = Some(ctx.waker().clone());
        Poll::Pending
    }
}

impl Default for Autex {
    fn default() -> Self {
        Autex::new()
    }
}

impl Autex {
    pub fn new() -> Self {
        let aux = Autex {
            hold: 0,
            mux: Mutex::new(Box::new(ListHead::new())),
        };
        aux.mux.lock().unwrap().init_list_head();
        aux
    }

    pub async fn lock<'a>(&'a self) -> Guard<'a> {
        let r = self
            .hold
            .atomic_compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed);
        if r.is_ok() {
            return Guard(self);
        }

        let mut autx = autex::new(self);
        let mut g0 = self.mux.lock().unwrap();
        unsafe {
            g0.list_add_tail(&mut autx.ent);
        }
        drop(g0);

        let g1 = (&mut autx).await;
        g0 = self.mux.lock().unwrap();
        unsafe {
            autx.ent.list_del();
        }
        drop(g0);
        g1
    }

    fn unlock(&self) {
        self.hold.atomic_store(0, Ordering::Relaxed);
        let mut g = self.mux.lock().unwrap();
        if g.list_empty() {
            return;
        }

        let autx = list_entry!(g.next, autex, ent);
        match autx.waker {
            Some(ref w) => {
                if 0 == self.hold.atomic_load(Ordering::Relaxed) {
                    // future will mux.lock after awake
                    // so unlock before waking it
                    drop(g);
                    w.wake_by_ref();
                }
            }
            _ => unreachable!(),
        }
    }
}
