use crate::{atomic::AtomicN, list_entry, ListHead};
use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::Ordering, Arc, Mutex},
    task::{self, Context, Poll, Wake, Waker},
};

pub struct Guard<'a>(pub(super) &'a Autex);
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
unsafe impl<'a> Send for autex<'a> {}

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

    fn wait_enter(&self, autx: &mut autex) {
        let mut g = self.mux.lock().unwrap();
        unsafe {
            g.list_add_tail(&mut autx.ent);
        }
        drop(g);
    }

    fn wait_leave(&self, autx: &mut autex) {
        let g = self.mux.lock().unwrap();
        unsafe {
            autx.ent.list_del();
        }
        drop(g);
    }

    pub async fn lock<'a>(&'a self) -> Guard<'a> {
        let r = self
            .hold
            .atomic_compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed);
        if r.is_ok() {
            return Guard(self);
        }

        let mut autx = autex::new(self);
        self.wait_enter(&mut autx);
        let g = (&mut autx).await;
        self.wait_leave(&mut autx);
        g
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
