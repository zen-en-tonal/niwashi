pub mod destroy_function;
pub mod fragile;

#[allow(unused_imports)]
pub use destroy_function::*;
pub use fragile::*;

pub trait Destroyer<T> {
    fn destroy(&self, other: impl AsRef<T>);
}

pub trait Corruptible {
    fn corrupt(self);
}

#[cfg(test)]
mod tests {
    use crate::destroy::*;
    use crate::mutation::*;

    fn increment(value: i32) -> i32 {
        value + 1
    }

    fn validate(value: &i32) -> bool {
        *value < 1
    }

    fn destroy(_: &i32) {}

    #[test]
    fn make_fragile() {
        let mutable = increment.make_mutable(0);
        let x = destroy.make_fragile(validate, mutable);
        assert_eq!(x.mutate().is_none(), true);
    }
}
