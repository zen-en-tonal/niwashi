pub mod introspective;
pub mod stop_growing;
pub mod validation_function;

pub use introspective::*;
pub use stop_growing::*;
#[allow(unused_imports)]
pub use validation_function::*;

pub trait Validator<T>: Sized {
    fn is_valid(&self, other: &impl AsRef<T>) -> bool;
}

pub trait Validatable {
    fn is_valid(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use crate::mutation::*;
    use crate::validation::*;

    fn increment(value: i32) -> i32 {
        value + 1
    }

    fn validate(value: &i32) -> bool {
        *value < 1
    }

    #[test]
    fn make_interspective() {
        let mutable = increment.make_mutable(0);
        let introspective = validate.make_introspective(mutable);
        assert_eq!(introspective.mutate().is_none(), true)
    }

    #[test]
    fn make_stop_growing() {
        let mutable = increment.make_mutable(0);
        let stop_growing = validate.make_stop_growing(mutable);
        assert_eq!(*stop_growing.mutate().unwrap().inner(), 0)
    }
}
