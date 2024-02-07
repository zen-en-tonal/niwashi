use super::Validator;

impl<F, T> Validator<T> for F
where
    F: Fn(&T) -> bool,
{
    fn is_valid(&self, other: &impl AsRef<T>) -> bool {
        (self)(other.as_ref())
    }
}
