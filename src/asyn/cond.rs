use super::{
    autex::{Autex, Guard},
    context,
};
use crate::{
    atomic::AtomicN, callpos, container_of, lkf_get, lkf_next_unsafe, nil, ListHead, Lkf, LkfNode,
};
use std::{
    future::Future,
    pin::Pin,
    sync::atomic::Ordering,
    task::{Context, Poll, Waker},
    thread,
};

pub struct Cond {
    wait_list: Lkf,
}

impl Cond {
    pub fn new() -> Self {
        Cond {
            wait_list: Lkf::new(),
        }
    }

    pub fn notify_one(&self) {
        let node = unsafe { self.wait_list.get_one() };
        if node == nil!() {
            return;
        }
        let wn = container_of!(node, WakeNode, node);
        wn.waked.atomic_store(1, Ordering::Relaxed);
        wn.waker.wake_by_ref();
    }

    pub fn notify_all(&self) {
        let nodes = lkf_get!(self.wait_list);
        if nodes == nil!() {
            return;
        }

        loop {
            let node = lkf_next_unsafe!(nodes);
            if node == nil!() {
                continue;
            }

            let wn = container_of!(node, WakeNode, node);
            wn.waked.atomic_store(1, Ordering::Relaxed);
            wn.waker.wake_by_ref();
            if node == nodes {
                break;
            }
        }
    }

    pub async fn wait<'a, 'b>(&'a self, g: Guard<'b>) -> Guard<'b> {
        let autx = g.0;
        let mut wn = WakeNode::new(context.await);
        unsafe {
            self.wait_list.put(&mut wn.node, callpos!());
        }
        drop(g);
        wn.await;
        autx.lock().await
    }
}

struct WakeNode {
    waked: i32,
    node: LkfNode,
    waker: Waker,
}

impl WakeNode {
    pub fn new(waker: Waker) -> Self {
        WakeNode {
            waked: 0,
            node: LkfNode::new(),
            waker: waker,
        }
    }
}

impl Future for WakeNode {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let waked = self.waked.atomic_load(Ordering::Relaxed);
        if waked == 0 {
            return Poll::Pending;
        }
        Poll::Ready(())
    }
}
