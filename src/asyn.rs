use std::{
    future::{Future, Ready},
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub mod autex;
pub mod cond;

struct context;
impl Future for context {
    type Output = Waker;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Waker> {
        Poll::Ready(ctx.waker().clone())
    }
}
