use std::marker::PhantomData;

use crate::mutation::Mutable;

use super::{Validatable, Validator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StopGrowing<T, Q, R>(T, Q, PhantomData<R>);

impl<T, Q, R> StopGrowing<T, Q, R> {
    pub(super) fn new(t: T, q: Q) -> StopGrowing<T, Q, R> {
        StopGrowing(t, q, PhantomData)
    }
}

impl<T, Q: AsRef<R>, R> StopGrowing<T, Q, R> {
    pub fn inner(&self) -> &R {
        self.as_ref()
    }
}

impl<T: Validator<R>, Q: Mutable + AsRef<R> + Clone, R> Mutable for StopGrowing<T, Q, R> {
    /// validationに失敗した場合は前状態を返す。
    fn mutate(self) -> Option<Self> {
        match self.1.clone().mutate() {
            Some(x) => match self.0.is_valid(&x) {
                true => Some(StopGrowing::new(self.0, x)),
                false => Some(StopGrowing::new(self.0, self.1)),
            },
            None => None,
        }
    }
}

impl<T: Validator<R>, Q: AsRef<R>, R> Validatable for StopGrowing<T, Q, R> {
    fn is_valid(&self) -> bool {
        self.0.is_valid(&self.1)
    }
}

impl<T, Q: AsRef<R>, R> AsRef<R> for StopGrowing<T, Q, R> {
    fn as_ref(&self) -> &R {
        self.1.as_ref()
    }
}

pub trait MakeStopGrowing<T, Q>: Sized {
    fn make_stop_growing(self, v: T) -> StopGrowing<Self, T, Q>;
}

impl<T: Validator<R>, Q: Mutable + AsRef<R>, R> MakeStopGrowing<Q, R> for T {
    fn make_stop_growing(self, v: Q) -> StopGrowing<Self, Q, R> {
        StopGrowing::new(self, v)
    }
}
