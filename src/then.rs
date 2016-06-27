use std::sync::Arc;

use {Future, IntoFuture, PollError, Wake, PollResult, Tokens};
use util;
use chain::Chain;

/// Future for the `then` combinator, chaining computations on the end of
/// another future regardless of its outcome.
///
/// This is created by this `Future::then` method.
pub struct Then<A, B, F, T, E, U, V>
    where A: Future<T, E>,
          B: IntoFuture<U, V>,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
          V: Send + 'static,
{
    state: Chain<A, B::Future, F, T, E, U, V>,
}

pub fn new<A, B, F, T, E, U, V>(future: A, f: F) -> Then<A, B, F, T, E, U, V>
    where A: Future<T, E>,
          B: IntoFuture<U, V>,
          F: FnOnce(Result<T, E>) -> B + Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
          V: Send + 'static,
{
    Then {
        state: Chain::new(future, f),
    }
}

impl<A, B, F, T, E, U, V> Future<U, V> for Then<A, B, F, T, E, U, V>
    where A: Future<T, E>,
          B: IntoFuture<U, V>,
          F: FnOnce(Result<T, E>) -> B + Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
          U: Send + 'static,
          V: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<U, V>> {
        self.state.poll(tokens, |a, f| {
            let ret = match a {
                Ok(e) => util::recover(|| f(Ok(e))),
                Err(PollError::Other(e)) => util::recover(|| f(Err(e))),
                Err(PollError::Panicked(e)) => Err(PollError::Panicked(e)),
            };
            ret.map(|b| Err(b.into_future()))
        })
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens  {
        self.state.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<U, V>>> {
        self.state.tailcall()
    }
}
