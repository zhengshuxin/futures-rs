use {Future, Poll, Task};

impl<T, E> Future for Box<Future<Item=T, Error=E>>
    where T: 'static,
          E: 'static,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self, task: &mut Task) -> Poll<Self::Item, Self::Error> {
        (**self).poll(task)
    }

    fn schedule(&mut self, task: &mut Task) {
        (**self).schedule(task)
    }
}

impl<T, E> Future for Box<Future<Item=T, Error=E> + Send>
    where T: Send + 'static,
          E: Send + 'static,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self, task: &mut Task) -> Poll<Self::Item, Self::Error> {
        (**self).poll(task)
    }

    fn schedule(&mut self, task: &mut Task) {
        (**self).schedule(task)
    }
}

impl<F: Future> Future for Box<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self, task: &mut Task) -> Poll<Self::Item, Self::Error> {
        (**self).poll(task)
    }

    fn schedule(&mut self, task: &mut Task) {
        (**self).schedule(task)
    }
}
