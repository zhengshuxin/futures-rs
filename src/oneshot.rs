use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use {Future, Task, Poll};
use slot::{Slot, Token};

/// A future representing the completion of a computation happening elsewhere in
/// memory.
///
/// This is created by the `oneshot` function.
pub struct Oneshot<'a, T>
    where T: Send + 'a,
{
    inner: Arc<Inner<'a, T>>,
    cancel_token: Option<Token>,
}

#[doc(hidden)]
#[deprecated(note = "renamed to `oneshot`")]
pub type Promise<T> = Oneshot<'static, T>;

/// Represents the completion half of a oneshot through which the result of a
/// computation is signaled.
///
/// This is created by the `oneshot` function.
pub struct Complete<'a, T>
    where T: Send + 'a,
{
    inner: Arc<Inner<'a, T>>,
    completed: bool,
}

struct Inner<'a, T> {
    slot: Slot<'a, Option<T>>,
    pending_wake: AtomicBool,
}

/// Creates a new in-memory oneshot used to represent completing a computation.
///
/// A oneshot in this library is a concrete implementation of the `Future` trait
/// used to complete a computation from one location with a future representing
/// what to do in another.
///
/// This function is similar to Rust's channels found in the standard library.
/// Two halves are returned, the first of which is a `Oneshot` which implements
/// the `Future` trait. The second half is a `Complete` handle which is used to
/// signal the end of a computation.
///
/// Each half can be separately owned and sent across threads.
///
/// # Examples
///
/// ```
/// use futures::*;
///
/// let (c, p) = oneshot::<i32>();
///
/// p.map(|i| {
///     println!("got: {}", i);
/// }).forget();
///
/// c.complete(3);
/// ```
pub fn oneshot<'a, T>() -> (Complete<'a, T>, Oneshot<'a, T>)
    where T: Send + 'a,
{
    let inner = Arc::new(Inner {
        slot: Slot::new(None),
        pending_wake: AtomicBool::new(false),
    });
    let oneshot = Oneshot {
        inner: inner.clone(),
        cancel_token: None,
    };
    let complete = Complete {
        inner: inner,
        completed: false,
    };
    (complete, oneshot)
}

#[doc(hidden)]
#[deprecated(note = "renamed to `oneshot`")]
pub fn promise<T>() -> (Complete<'static, T>, Oneshot<'static, T>)
    where T: Send + 'static,
{
    oneshot()
}

impl<'b, T> Complete<'b, T>
    where T: Send + 'b,
{
    /// Completes this oneshot with a successful result.
    ///
    /// This function will consume `self` and indicate to the other end, the
    /// `Oneshot`, that the error provided is the result of the computation this
    /// represents.
    pub fn complete(mut self, t: T) {
        self.completed = true;
        self.send(Some(t))
    }

    fn send(&mut self, t: Option<T>) {
        if let Err(e) = self.inner.slot.try_produce(t) {
            self.inner.slot.on_empty(|slot| {
                slot.try_produce(e.into_inner()).ok()
                    .expect("advertised as empty but wasn't");
            });
        }
    }
}

impl<'b, T> Drop for Complete<'b, T>
    where T: Send + 'b,
{
    fn drop(&mut self) {
        if !self.completed {
            self.send(None);
        }
    }
}

/// Error returned from a `Oneshot<T>` whenever the correponding `Complete<T>`
/// is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl<'b, T: Send + 'b> Future for Oneshot<'b, T> {
    type Item = T;
    type Error = Canceled;

    fn poll<'a>(&mut self, _: &mut Task<'a>) -> Poll<T, Canceled>
        where Self: 'a,
    {
        if self.inner.pending_wake.load(Ordering::SeqCst) {
            return Poll::NotReady
        }
        match self.inner.slot.try_consume() {
            Ok(Some(e)) => Poll::Ok(e),
            Ok(None) => Poll::Err(Canceled),
            Err(_) => Poll::NotReady,
        }
    }

    fn schedule<'a>(&mut self, task: &mut Task<'a>)
        where Self: 'a,
    {
        if self.inner.pending_wake.load(Ordering::SeqCst) {
            if let Some(cancel_token) = self.cancel_token.take() {
                self.inner.slot.cancel(cancel_token);
            }
        }
        self.inner.pending_wake.store(true, Ordering::SeqCst);
        let inner = self.inner.clone();
        let handle = task.handle().clone();
        self.cancel_token = Some(self.inner.slot.on_full(move |_| {
            inner.pending_wake.store(false, Ordering::SeqCst);
            handle.notify();
        }));
    }
}

impl<'b, T> Drop for Oneshot<'b, T>
    where T: Send + 'b,
{
    fn drop(&mut self) {
        if let Some(cancel_token) = self.cancel_token.take() {
            self.inner.slot.cancel(cancel_token)
        }
    }
}
