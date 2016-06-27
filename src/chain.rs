use std::marker;
use std::mem;
use std::sync::Arc;

use util::{self, Collapsed};
use {Future, PollResult, Wake, Tokens};

pub enum Chain<A, B, C, T, E, U, F>
    where A: Future<T, E>,
          B: Future<U, F>,
          T: Send + 'static,
          U: Send + 'static,
          E: Send + 'static,
          F: Send + 'static,
{
    First(Collapsed<A, T, E>, C),
    Second(B, marker::PhantomData<fn() -> (U, F)>),
    Done,
}

impl<A, B, C, T, U, E, F> Chain<A, B, C, T, E, U, F>
    where A: Future<T, E>,
          B: Future<U, F>,
          C: Send + 'static,
          T: Send + 'static,
          U: Send + 'static,
          E: Send + 'static,
          F: Send + 'static,
{
    pub fn new(a: A, c: C) -> Chain<A, B, C, T, E, U, F> {
        Chain::First(Collapsed::Start(a), c)
    }

    pub fn poll<G>(&mut self, tokens: &Tokens, f: G) -> Option<PollResult<U, F>>
        where G: FnOnce(PollResult<T, E>, C)
                        -> PollResult<Result<U, B>, F> + Send + 'static,
    {
        let a_result = match *self {
            Chain::First(ref mut a, _) => {
                match a.poll(tokens) {
                    Some(a) => a,
                    None => return None,
                }
            }
            Chain::Second(ref mut b, _) => return b.poll(tokens),
            Chain::Done => return Some(Err(util::reused())),
        };
        let data = match mem::replace(self, Chain::Done) {
            Chain::First(_, c) => c,
            _ => panic!(),
        };
        match f(a_result, data) {
            Ok(Ok(e)) => Some(Ok(e)),
            Ok(Err(mut b)) => {
                let ret = b.poll(&Tokens::all());
                *self = Chain::Second(b, marker::PhantomData);
                ret
            }
            Err(e) => Some(Err(e)),
        }
    }

    pub fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        match *self {
            Chain::First(ref mut a, _) => a.schedule(wake),
            Chain::Second(ref mut b, _) => b.schedule(wake),
            Chain::Done => util::done(wake),
        }
    }

    pub fn tailcall(&mut self) -> Option<Box<Future<U, F>>> {
        match *self {
            Chain::First(ref mut a, _) => {
                a.collapse();
                None
            }
            Chain::Second(ref mut b, _) => b.tailcall(),
            Chain::Done => None,
        }
    }
}
