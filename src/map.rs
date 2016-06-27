use std::sync::Arc;

use {Future, PollResult, Wake, Tokens};
use util::{self, Collapsed};

/// Future for the `map` combinator, changing the type of a future.
///
/// This is created by this `Future::map` method.
pub struct Map<A, F, T, E>
    where A: Future<T, E>,
          T: Send + 'static,
          E: Send + 'static,
{
    future: Collapsed<A, T, E>,
    f: Option<F>,
}

pub fn new<U, A, F, T, E>(future: A, f: F) -> Map<A, F, T, E>
    where A: Future<T, E>,
          F: FnOnce(T) -> U + Send + 'static,
          U: Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
{
    Map {
        future: Collapsed::Start(future),
        f: Some(f),
    }
}

impl<U, A, F, T, E> Future<U, E> for Map<A, F, T, E>
    where A: Future<T, E>,
          F: FnOnce(T) -> U + Send + 'static,
          U: Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<U, E>> {
        let result = match self.future.poll(tokens) {
            Some(result) => result,
            None => return None,
        };
        let callback = util::opt2poll(self.f.take());
        Some(result.and_then(|e| {
            callback.map(|f| f(e))
        }))
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.future.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<U, E>>> {
        self.future.collapse();
        None
    }
}
