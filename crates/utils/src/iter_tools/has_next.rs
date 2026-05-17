use std::iter::Peekable;

pub trait HasNext {
    fn has_next(&mut self) -> bool;
}

impl<I> HasNext for Peekable<I>
where
    I: Iterator,
{
    fn has_next(&mut self) -> bool {
        self.peek().is_some()
    }
}
