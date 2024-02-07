use self::mutable::IntoMutable;

pub mod mutable;
pub mod mutation_function;

#[allow(unused_imports)]
pub use mutable::*;
#[allow(unused_imports)]
pub use mutation_function::*;

pub trait Mutator<T>: Sized {
    fn mutation(&self, from: impl Into<T>) -> Option<T>;

    fn make_mutable(self, v: impl Into<T>) -> IntoMutable<Self, T> {
        IntoMutable::new(self, v.into())
    }
}

pub trait Mutable<T = Self>: Sized {
    fn mutate(self) -> Option<T>;
}

#[cfg(test)]
mod tests {
    use crate::mutation::*;

    fn increment(value: i32) -> i32 {
        value + 1
    }

    #[test]
    fn make_mutable() {
        let mutable = increment.make_mutable(0);
        assert_eq!(*mutable.mutate().unwrap().inner(), 1)
    }
}
