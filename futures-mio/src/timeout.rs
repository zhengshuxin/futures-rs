use std::io;
use std::time::{Duration, Instant};

use futures::{Future, Task, Poll};
use futures_io::IoFuture;

use LoopHandle;
use event_loop::TimeoutToken;

/// A future representing the notification that a timeout has occurred.
///
/// Timeouts are created through the `LoopHandle::timeout` or
/// `LoopHandle::timeout_at` methods indicating when a timeout should fire at.
/// Note that timeouts are not intended for high resolution timers, but rather
/// they will likely fire some granularity after the exact instant that they're
/// otherwise indicated to fire at.
pub struct Timeout {
    at: Instant,
    token: TimeoutToken,
    handle: LoopHandle,
}

impl LoopHandle {
    /// Creates a new timeout which will fire at `dur` time into the future.
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn timeout(self, dur: Duration) -> IoFuture<Timeout> {
        self.timeout_at(Instant::now() + dur)
    }

    /// Creates a new timeout which will fire at the time specified by `at`.
    ///
    /// This function will return a future that will resolve to the actual
    /// timeout object. The timeout object itself is then a future which will be
    /// set to fire at the specified point in the future.
    pub fn timeout_at(self, at: Instant) -> IoFuture<Timeout> {
        self.add_timeout(at).map(move |token| {
            Timeout {
                at: at,
                token: token,
                handle: self,
            }
        }).boxed()
    }
}

impl Future for Timeout {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self, _task: &mut Task) -> Poll<(), io::Error> {
        // TODO: is this fast enough?
        if self.at <= Instant::now() {
            Poll::Ok(())
        } else {
            Poll::NotReady
        }
    }

    fn schedule(&mut self, task: &mut Task) {
        self.handle.update_timeout(&self.token, task);
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        self.handle.cancel_timeout(&self.token);
    }
}
