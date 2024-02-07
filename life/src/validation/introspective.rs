use std::marker::PhantomData;

use crate::mutation::Mutable;

use super::{Validatable, Validator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 内省的なMutable
pub struct Introspective<T, Q, R>(T, Q, PhantomData<R>);

impl<T, Q, R> Introspective<T, Q, R> {
    pub(super) fn new(t: T, q: Q) -> Introspective<T, Q, R> {
        Introspective(t, q, PhantomData)
    }
}

impl<T, Q: AsRef<R>, R> Introspective<T, Q, R> {
    pub fn inner(&self) -> &R {
        self.as_ref()
    }
}

impl<T: Validator<R>, Q: Mutable + AsRef<R>, R> Mutable for Introspective<T, Q, R> {
    /// validationに失敗した場合はNoneを返す。
    fn mutate(self) -> Option<Self> {
        match self.1.mutate() {
            Some(x) => match self.0.is_valid(&x) {
                true => Some(Introspective::new(self.0, x)),
                false => None,
            },
            None => None,
        }
    }
}

impl<T: Validator<R>, Q: AsRef<R>, R> Validatable for Introspective<T, Q, R> {
    fn is_valid(&self) -> bool {
        self.0.is_valid(&self.1)
    }
}

impl<T, Q: AsRef<R>, R> AsRef<R> for Introspective<T, Q, R> {
    fn as_ref(&self) -> &R {
        self.1.as_ref()
    }
}

pub trait MakeIntrospective<T, Q>: Sized {
    fn make_introspective(self, v: T) -> Introspective<Self, T, Q>;
}

impl<T: Validator<R>, Q: Mutable + AsRef<R>, R> MakeIntrospective<Q, R> for T {
    fn make_introspective(self, v: Q) -> Introspective<Self, Q, R> {
        Introspective::new(self, v)
    }
}
