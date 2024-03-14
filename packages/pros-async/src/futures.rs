//! Commonly used futures designed for the pros-rs async runtime.

use core::{future::Future, task::Poll};

use crate::executor::EXECUTOR;

/// A future that will complete after the given duration.
/// Sleep futures that are closer to completion are prioritized to improve accuracy.
#[derive(Debug)]
pub struct SleepFuture {
    target_millis: u32,
}
impl Future for SleepFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.target_millis < unsafe { pros_sys::millis() } {
            Poll::Ready(())
        } else {
            EXECUTOR.with(|e| {
                e.reactor
                    .borrow_mut()
                    .sleepers
                    .push(cx.waker().clone(), self.target_millis)
            });
            Poll::Pending
        }
    }
}

/// Returns a future that will complete after the given duration.
pub fn sleep(duration: core::time::Duration) -> SleepFuture {
    SleepFuture {
        target_millis: unsafe { pros_sys::millis() + duration.as_millis() as u32 },
    }
}

#[derive(Debug)]
/// A future that completes once a predicate returns true.
pub struct WaitUntilFuture<F: Fn() -> bool> {
    predicate: F,
}
impl<F: Fn() -> bool> Future for WaitUntilFuture<F> {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if (self.predicate)() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Returns a future that completes once the given predicate returns true.
pub fn wait_until<F: Fn() -> bool>(predicate: F) -> WaitUntilFuture<F> {
    WaitUntilFuture { predicate }
}
