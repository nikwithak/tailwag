use serde::Serialize;

pub trait Webhook<I, R> {
    fn run() -> R;
}

impl<F, I, R> Webhook<I, R> for F
where
    F: Fn(I) -> R, // The function being inferred. Need to define I in terms of extractable traits.
    R: Serialize,  // Response Type.
    I:,
{
    fn run() -> R {
        todo!()
    }
}
