pub trait Mutator<T>: Sized {
    fn mutation<R: Into<T>>(&self, from: R) -> Option<T>;
}

pub trait Mutable<T = Self>: Sized {
    fn mutate(self) -> Option<T>;
}
