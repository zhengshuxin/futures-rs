use {Future, Poll, Task};

impl<F: Future + ?Sized> Future for Box<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self, task: &mut Task) -> Poll<Self::Item, Self::Error> {
        (**self).poll(task)
    }

    fn schedule(&mut self, task: &mut Task) {
        (**self).schedule(task)
    }
}
