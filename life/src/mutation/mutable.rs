use super::{Mutable, Mutator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntoMutable<T, Q>(T, Q);

impl<T, Q> IntoMutable<T, Q> {
    pub fn inner(&self) -> &Q {
        &self.1
    }
}

impl<T, Q> IntoMutable<T, Q> {
    pub(super) fn new(t: T, q: Q) -> IntoMutable<T, Q> {
        IntoMutable(t, q)
    }
}

impl<T, Q> AsRef<Q> for IntoMutable<T, Q> {
    fn as_ref(&self) -> &Q {
        self.inner()
    }
}

impl<T, Q> AsMut<Q> for IntoMutable<T, Q> {
    fn as_mut(&mut self) -> &mut Q {
        &mut self.1
    }
}

impl<T: Mutator<Q>, Q> Mutable for IntoMutable<T, Q> {
    fn mutate(self) -> Option<Self> {
        match self.0.mutation(self.1) {
            Some(x) => Some(IntoMutable::new(self.0, x)),
            None => None,
        }
    }
}
