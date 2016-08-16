use {Future, Poll, Task};

impl<F: Future + ?Sized> Future for Box<F> {
    type Item = F::Item;
    type Error = F::Error;

    // TODO: um... is this sound?
    fn poll<'a>(&mut self, task: &mut Task<'a>) -> Poll<Self::Item, Self::Error>
        where F: 'a
    {
        (**self).poll(task)
    }

    fn schedule<'a>(&mut self, task: &mut Task<'a>)
        where F: 'a
    {
        (**self).schedule(task)
    }
}
