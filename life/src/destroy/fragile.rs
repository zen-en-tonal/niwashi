use std::marker::PhantomData;

use crate::{
    mutation::Mutable,
    validation::{Validatable, Validator},
};

use super::{Corruptible, Destroyer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fragile<T, Q, R, S>(T, Q, R, PhantomData<S>);

impl<T, Q, R, S> Fragile<T, Q, R, S> {
    pub(super) fn new(t: T, q: Q, r: R) -> Fragile<T, Q, R, S> {
        Fragile(t, q, r, PhantomData)
    }
}

impl<T, Q, R, S> Fragile<T, Q, R, S>
where
    R: AsRef<S>,
{
    pub fn inner(&self) -> &S {
        self.as_ref()
    }
}

impl<T, Q, R, S> Mutable for Fragile<T, Q, R, S>
where
    T: Destroyer<S>,
    Q: Validator<S>,
    R: Mutable + AsRef<S> + Clone,
{
    fn mutate(self) -> Option<Self> {
        match self.2.clone().mutate() {
            Some(ok) => match self.1.is_valid(&ok) {
                true => Some(Fragile::new(self.0, self.1, ok)),
                false => {
                    self.corrupt();
                    None
                }
            },
            None => None,
        }
    }
}

impl<T, Q, R, S> Validatable for Fragile<T, Q, R, S>
where
    T: Destroyer<S>,
    Q: Validator<S>,
    R: Mutable + AsRef<S>,
{
    fn is_valid(&self) -> bool {
        self.1.is_valid(&self.2)
    }
}

impl<T, Q, R, S> Corruptible for Fragile<T, Q, R, S>
where
    T: Destroyer<S>,
    Q: Validator<S>,
    R: Mutable + AsRef<S>,
{
    fn corrupt(self) {
        self.0.destroy(self.2)
    }
}

impl<T, Q, R, S> AsRef<S> for Fragile<T, Q, R, S>
where
    R: AsRef<S>,
{
    fn as_ref(&self) -> &S {
        self.2.as_ref()
    }
}

pub trait MakeFragile<T, Q, R>: Sized {
    fn make_fragile(self, valid: T, value: Q) -> Fragile<Self, T, Q, R>;
}

impl<T, Q, R, S> MakeFragile<Q, R, S> for T
where
    T: Destroyer<S>,
    Q: Validator<S>,
    R: Mutable + AsRef<S>,
{
    fn make_fragile(self, valid: Q, value: R) -> Fragile<Self, Q, R, S> {
        Fragile::new(self, valid, value)
    }
}
