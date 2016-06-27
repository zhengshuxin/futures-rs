use std::sync::Arc;

use {Future, PollResult, Wake, Tokens};
use util::{self, Collapsed};

/// Future for the `map_err` combinator, changing the error type of a future.
///
/// This is created by this `Future::map_err` method.
pub struct MapErr<A, F, T, E>
    where A: Future<T, E>,
          T: Send + 'static,
          E: Send + 'static,
{
    future: Collapsed<A, T, E>,
    f: Option<F>,
}

pub fn new<U, A, F, T, E>(future: A, f: F) -> MapErr<A, F, T, E>
    where A: Future<T, E>,
          F: FnOnce(E) -> U + Send + 'static,
          U: Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
{
    MapErr {
        future: Collapsed::Start(future),
        f: Some(f),
    }
}

impl<U, A, F, T, E> Future<T, U> for MapErr<A, F, T, E>
    where A: Future<T, E>,
          F: FnOnce(E) -> U + Send + 'static,
          U: Send + 'static,
          T: Send + 'static,
          E: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<T, U>> {
        let result = match self.future.poll(tokens) {
            Some(result) => result,
            None => return None,
        };
        let f = util::opt2poll(self.f.take());
        Some(f.and_then(|f| {
            result.map_err(|e| e.map(f))
        }))
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.future.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<T, U>>> {
        self.future.collapse();
        None
    }
}
