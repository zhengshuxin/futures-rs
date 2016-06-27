use std::sync::Arc;

use {Future, IntoFuture, Wake, PollResult, Tokens};
use chain::Chain;

/// Future for the `flatten` combinator, flattening a future-of-a-future to just
/// the result of the final future.
///
/// This is created by this `Future::flatten` method.
pub struct Flatten<A, T, E, U>
    where A: Future<T, E>,
          T: IntoFuture<U, E>,
          E: Send + 'static,
          U: Send + 'static,
{

    state: Chain<A, T::Future, (), T, E, U, E>,
}

pub fn new<A, T, E, U>(future: A) -> Flatten<A, T, E, U>
    where A: Future<T, E>,
          T: IntoFuture<U, E>,
          E: Send + 'static,
          U: Send + 'static,
{
    Flatten {
        state: Chain::new(future, ()),
    }
}

impl<A, T, E, U> Future<U, E> for Flatten<A, T, E, U>
    where A: Future<T, E>,
          T: IntoFuture<U, E>,
          E: Send + 'static,
          U: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<U, E>> {
        self.state.poll(tokens, |a, ()| {
            match a {
                Ok(item) => Ok(Err(item.into_future())),
                Err(e) => Err(e.map(From::from)),
            }
        })
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.state.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<U, E>>> {
        self.state.tailcall()
    }
}
