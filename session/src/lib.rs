use death::Motal;
use pool::Store;
use transions::Mutable;
use validations::Validator;

pub mod death;
pub mod pool;
pub mod transions;
pub mod validations;

pub fn mutate_in_pool<K, V: Mutable>(set: &impl Store<K, V>, key: &K) -> Option<V> {
    match set.get(key) {
        Some(m) => m.mutate(),
        None => None,
    }
}

pub fn mutate_in_pool_with_validator<K, T, V: Mutable + AsRef<T>>(
    set: &impl Store<K, V>,
    val: &impl Validator<T>,
    key: &K,
) -> Option<V> {
    match set.get(key) {
        Some(m) => match m.mutate() {
            Some(a) => match val.is_valid(&a) {
                true => Some(a),
                false => None,
            },
            None => None,
        },
        None => None,
    }
}

pub fn mutate_in_pool_with_motal<K, T, V: Mutable + AsRef<T> + Motal>(
    set: &impl Store<K, V>,
    val: &impl Validator<T>,
    key: &K,
) -> Option<V> {
    match set.get(key) {
        Some(m) => match m.mutate() {
            Some(a) => match val.is_valid(&a) {
                true => Some(a),
                false => {
                    a.die();
                    None
                }
            },
            None => None,
        },
        None => None,
    }
}
