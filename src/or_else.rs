use std::sync::Arc;

use {Future, IntoFuture, Wake, PollResult, PollError, Tokens};
use chain::Chain;
use util;

/// Future for the `or_else` combinator, chaining a computation onto the end of
/// a future which fails with an error.
///
/// This is created by this `Future::or_else` method.
pub struct OrElse<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<T, U>,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
{
    state: Chain<A, B::Future, F, T, E, T, U>,
}

pub fn new<A, B, F, T, E, U>(future: A, f: F) -> OrElse<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<T, U>,
          F: FnOnce(E) -> B + Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
{
    OrElse {
        state: Chain::new(future, f),
    }
}

impl<A, B, F, T, E, U> Future<T, U> for OrElse<A, B, F, T, E, U>
    where A: Future<T, E>,
          B: IntoFuture<T, U>,
          F: FnOnce(E) -> B + Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<T, U>> {
        self.state.poll(tokens, |a, f| {
            match a {
                Ok(item) => Ok(Ok(item)),
                Err(PollError::Panicked(d)) => Err(PollError::Panicked(d)),
                Err(PollError::Other(e)) => {
                    util::recover(|| f(e)).map(|e| Err(e.into_future()))
                }
            }
        })
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.state.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<T, U>>> {
        self.state.tailcall()
    }
}
