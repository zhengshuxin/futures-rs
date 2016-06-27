use std::mem;
use std::sync::Arc;

use {Future, PollResult, Wake, Tokens, empty};

impl<T, E> Future<T, E> for Box<Future<T, E>>
    where T: Send + 'static,
          E: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<T, E>> {
        (**self).poll(tokens)
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        (**self).schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<T, E>>> {
        if let Some(f) = (**self).tailcall() {
            return Some(f)
        }
        Some(mem::replace(self, Box::new(empty())))
    }
}

impl<F, T, E> Future<T, E> for Box<F>
    where F: Future<T, E>,
          T: Send + 'static,
          E: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<T, E>> {
        (**self).poll(tokens)
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        (**self).schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<T, E>>> {
        (**self).tailcall()
    }
}
