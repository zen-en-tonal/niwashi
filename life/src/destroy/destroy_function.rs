use super::Destroyer;

impl<F, T> Destroyer<T> for F
where
    F: Fn(&T),
{
    fn destroy(&self, other: impl AsRef<T>) {
        (self)(other.as_ref())
    }
}
