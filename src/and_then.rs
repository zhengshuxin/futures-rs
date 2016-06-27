use std::sync::Arc;

use {Future, IntoFuture, Wake, PollResult, Tokens};
use util;
use chain::Chain;

/// Future for the `and_then` combinator, chaining a computation onto the end of
/// another future which completes successfully.
///
/// This is created by this `Future::and_then` method.
pub struct AndThen<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<U, E>,
          T: Send + 'static,
          U: Send + 'static,
          E: Send + 'static,
{
    state: Chain<A, B::Future, F, T, E, U, E>,
}

pub fn new<A, B, F, T, E, U>(future: A, f: F) -> AndThen<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<U, E>,
          F: FnOnce(T) -> B + Send + 'static,
          T: Send + 'static,
          U: Send + 'static,
          E: Send + 'static,
{
    AndThen {
        state: Chain::new(future, f),
    }
}

impl<A, B, F, T, E, U> Future<U, E> for AndThen<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<U, E>,
          F: FnOnce(T) -> B + Send + 'static,
          T: Send + 'static,
          U: Send + 'static,
          E: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<U, E>> {
        self.state.poll(tokens, |result, f| {
            let e = try!(result);
            util::recover(|| f(e)).map(|b| Err(b.into_future()))
        })
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.state.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<U, E>>> {
        self.state.tailcall()
    }
}
