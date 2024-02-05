pub trait Validator<T> {
    fn is_valid<R: AsRef<T>>(&self, other: &R) -> bool;
}

pub trait Validatable {
    fn is_valid(&self) -> bool;
}
