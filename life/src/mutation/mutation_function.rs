use super::Mutator;

impl<F, T> Mutator<T> for F
where
    F: Fn(T) -> T,
{
    fn mutation(&self, from: impl Into<T>) -> Option<T> {
        Some((self)(from.into()))
    }
}
